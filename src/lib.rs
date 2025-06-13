mod blog_repository;
mod cache;
mod config;
mod model;
use blog_repository::{BlogRepository, FileSystemBlogRepository, RepositoryError};
pub use config::BlogConfig;
use model::{format_date_for_post_view, format_date_for_posts_overview, BlogPost, Markdown};

use async_trait::async_trait;
use pulldown_cmark::{html, Options, Parser};
use shuttle_axum::axum::{
    extract::Path,
    http::StatusCode,
    response::Html,
    routing::{get, get_service},
    Extension, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

use crate::cache::CachedRenderer;

pub fn create_app_with_defaults() -> Router {
    create_app_with_dirs("content", "content")
}

pub fn create_app_with_dirs<P: Into<PathBuf> + Clone>(content_dir: P, blog_dir: P) -> Router {
    let content_dir = ContentDir(content_dir.into());
    let blog_dir = BlogDir(blog_dir.into());
    let config = BlogConfig::from_file_or_default(content_dir.config_file());
    create_app(content_dir.into(), &blog_dir, config)
}

// Files part of this project (a simple markdown based blog with fixed style).
pub struct BlogDir(pub PathBuf);

// Blog content
pub struct ContentDir(pub PathBuf);

impl BlogDir {
    pub fn dir(&self) -> PathBuf {
        self.0.clone()
    }
    fn templates_dir(&self) -> PathBuf {
        self.dir().join("templates")
    }
    fn static_dir(&self) -> PathBuf {
        self.dir().join("static")
    }
}

impl ContentDir {
    pub fn dir(&self) -> PathBuf {
        self.0.clone()
    }
    pub fn config_file(&self) -> PathBuf {
        self.dir().join("blog_config.yaml")
    }
}

fn create_app(content_dir: ContentDir, blog_dir: &BlogDir, config: BlogConfig) -> Router {
    let repo = create_repo(content_dir.dir());
    let blog_handler = BlogPostHandler::new(config, repo, blog_dir);

    let cached_renderer = CachedRenderer::new(Arc::new(blog_handler));
    let shared_renderer: Arc<dyn Renderer + Send + Sync> = Arc::new(cached_renderer);

    let statics = blog_dir.static_dir();
    let static_service = get_service(ServeDir::new(statics)).handle_error(|error| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error serving static file: {}", error),
        )
    });

    Router::new()
        .route("/health", get(|| async { "I'm ok!" }))
        .route("/", get(index_handler))
        .route("/p/{slug}", get(page_handler))
        .route("/{slug}", get(post_handler))
        .nest_service("/static", static_service)
        .layer(axum::extract::Extension(shared_renderer))
}

fn create_repo<P: Into<PathBuf> + Clone>(content_dir: P) -> FileSystemBlogRepository {
    FileSystemBlogRepository::new(content_dir.clone().into())
}

async fn index_handler(
    blog_handler: Extension<Arc<dyn Renderer + Send + Sync>>,
) -> Result<Html<String>, StatusCode> {
    blog_handler.0.posts().await
}

async fn page_handler(
    Path(slug): Path<String>,
    blog_handler: Extension<Arc<dyn Renderer + Send + Sync>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.page_for(slug).await?;
    Ok(html)
}

async fn post_handler(
    Path(slug): Path<String>,
    blog_handler: Extension<Arc<dyn Renderer + Send + Sync>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.post_for(slug).await?;
    Ok(html)
}

#[async_trait]
trait Renderer {
    async fn post_for(&self, slug: String) -> Result<Html<String>, StatusCode>;
    async fn page_for(&self, slug: String) -> Result<Html<String>, StatusCode>;
    async fn posts(&self) -> Result<Html<String>, StatusCode>;
}

type ThreadSafeBlogRepository = Arc<dyn BlogRepository + Send + Sync>;

#[derive(Clone)]
pub struct BlogPostHandler {
    repo: ThreadSafeBlogRepository,
    templates: Tera,
    config: BlogConfig,
}

impl BlogPostHandler {
    pub fn new(
        config: BlogConfig,
        blog_repo: impl BlogRepository + Send + Sync + 'static,
        blog_dir: &BlogDir,
    ) -> Self {
        let template_path = format!("{}{}", blog_dir.templates_dir().display(), "/**/*.html");
        let templates = match Tera::new(&template_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error(s): {}", e);
                Tera::default()
            }
        };

        let repo = Arc::new(blog_repo);

        Self {
            repo,
            templates,
            config,
        }
    }

    pub async fn render_posts(&self) -> Result<Html<String>, StatusCode> {
        let posts = self.get_all_posts()?;

        let mut context = self.build_base_context("/");
        context.insert("posts", &posts);

        self.templates
            .render("index.html", &context)
            .map_err(|e| {
                eprintln!("Template error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(Html)
    }

    fn get_all_posts(&self) -> Result<Vec<BlogPost>, StatusCode> {
        let markdowns = self.repo.get_all_posts().map_err(Self::into)?;

        let posts = markdowns
            .into_iter()
            .map(|markdown| BlogPost {
                title: markdown.title.clone().unwrap_or("Untitled".to_string()),
                publish_date: markdown.publish_date.map(format_date_for_posts_overview),
                slug: markdown.primary_slug(),
            })
            .collect();

        Ok(posts)
    }

    pub async fn render_page(&self, slug: String) -> Result<Html<String>, StatusCode> {
        log::info!("Requested page: {}", &slug);
        let page = self.repo.get_page(&slug).map_err(Self::into)?;
        let markdown = page.ok_or(StatusCode::NOT_FOUND)?;

        let mut context = self.build_base_context(&format!("/p/{}", slug));
        insert_content(&markdown, &mut context);
        insert_title(&markdown, &mut context);

        self.templates
            .render("page.html", &context)
            .map_err(|e| {
                eprintln!("Template error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(Html)
    }

    pub async fn render_post(&self, slug: String) -> Result<Html<String>, StatusCode> {
        let post = self.repo.find_post_by_slug(&slug).map_err(Self::into)?;
        let markdown = post.ok_or(StatusCode::NOT_FOUND)?;

        let mut context = self.build_base_context(&format!("/{}", slug));
        insert_content(&markdown, &mut context);
        insert_title(&markdown, &mut context);
        insert_published_date(markdown, &mut context);

        self.templates
            .render("post.html", &context)
            .map_err(|e| {
                eprintln!("Template error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
            .map(Html)
    }

    fn into(repo_err: RepositoryError) -> StatusCode {
        match repo_err {
            blog_repository::RepositoryError::NotFound => StatusCode::NOT_FOUND,
            blog_repository::RepositoryError::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn build_base_context(&self, path: &str) -> Context {
        let mut context = Context::new();

        let config = &self.config;

        let now = chrono::Local::now();
        context.insert("now", &now.to_rfc3339());
        context.insert("current_url", path);
        context.insert("site_title", &config.site_title);
        context.insert("site_description", &config.site_description);

        context
    }
}

#[async_trait]
impl Renderer for BlogPostHandler {
    async fn posts(&self) -> Result<Html<String>, StatusCode> {
        BlogPostHandler::render_posts(self).await
    }

    async fn post_for(&self, slug: String) -> Result<Html<String>, StatusCode> {
        BlogPostHandler::render_post(self, slug).await
    }

    async fn page_for(&self, slug: String) -> Result<Html<String>, StatusCode> {
        BlogPostHandler::render_page(self, slug).await
    }
}

fn insert_content(markdown: &Markdown, context: &mut Context) {
    let html_content = parse_to_html(markdown);
    context.insert("content", &html_content);
}

fn insert_title(markdown: &Markdown, context: &mut Context) {
    if let Some(title) = &markdown.title {
        context.insert("title", &title);
    }
}

fn insert_published_date(markdown: Markdown, context: &mut Context) {
    if let Some(date_str) = &markdown.publish_date {
        let formatted_date = format_date_for_post_view(*date_str);
        context.insert("date", &formatted_date);
    }
}

fn parse_to_html(markdown: &Markdown) -> String {
    let options = Options::empty();
    let parser = Parser::new_ext(&markdown.content, options);
    let mut html_content = String::new();
    html::push_html(&mut html_content, parser);
    html_content
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_date_str(date_str: &str) -> String {
        if let Some(date) = model::parse_date_for_sorting(date_str) {
            return format_date_for_post_view(date);
        }
        date_str.to_string()
    }

    #[test]
    fn test_format_date_js_format() {
        // JavaScript date format
        let input = "Fri Dec 06 2024 12:36:53 GMT+0000 (Coordinated Universal Time)";
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_js_format_without_timezone_name() {
        // JavaScript date format without timezone name in parentheses
        let input = "Fri Dec 06 2024 12:36:53 GMT+0000";
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_iso_format() {
        // Simple ISO format
        let input = "2024-12-06";
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_invalid_format() {
        // Invalid format should return the original string
        let input = "Invalid date";
        let expected = "Invalid date";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_mixed_format() {
        // Another common format
        let input = "Dec 6, 2024";

        // With our new implementation, this format is now supported
        let expected = "December 06, 2024";
        assert_eq!(format_date_str(input), expected);
    }

    #[test]
    fn test_format_date_consistency() {
        let test_dates = [
            "2023-01-15",
            "Fri Dec 06 2024 12:36:53 GMT+0000",
            "Mar 10, 2023",
            "January 20, 2024",
        ];

        for date in test_dates {
            let formatted = format_date_str(date);
            assert!(model::parse_date_for_sorting(&formatted).is_some());
        }
    }
}
