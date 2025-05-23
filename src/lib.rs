mod blog_repository;
mod cache;
mod config;
use blog_repository::{BlogRepository, FileSystemBlogRepository, RepositoryError};
use cache::CachedBlogRepository;
pub use config::BlogConfig;

use gray_matter::engine::YAML;
use gray_matter::Matter;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
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

pub fn create_app_with_defaults() -> Router {
    create_app_with_dirs("content", "content")
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

pub fn create_app_with_dirs<P: Into<PathBuf> + Clone>(content_dir: P, blog_dir: P) -> Router {
    let content_dir = ContentDir(content_dir.into());
    let blog_dir = BlogDir(blog_dir.into());
    let config = BlogConfig::from_file_or_default(content_dir.config_file());
    create_app(content_dir.into(), &blog_dir, config)
}

fn create_app(content_dir: ContentDir, blog_dir: &BlogDir, config: BlogConfig) -> Router {
    let repo = create_repo(content_dir.dir());
    let blog_handler = Arc::new(BlogPostHandler::new(config, repo, blog_dir));

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
        .layer(axum::extract::Extension(blog_handler))
}

fn create_repo<P: Into<PathBuf> + Clone>(
    content_dir: P,
) -> CachedBlogRepository<FileSystemBlogRepository> {
    let file_system_repo = FileSystemBlogRepository::new(content_dir.clone().into());
    let mut cached_repo = CachedBlogRepository::new(file_system_repo);
    if let Err(e) = cached_repo.refresh() {
        log::error!("Failed to populate initial cache: {:?}", e);
    }
    let repo = cached_repo;
    repo
}

async fn index_handler(
    blog_handler: Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.list_posts().await?;
    Ok(Html(html))
}

async fn page_handler(
    Path(slug): Path<String>,
    blog_handler: Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.render_page(slug).await?;
    Ok(Html(html))
}

async fn post_handler(
    Path(slug): Path<String>,
    blog_handler: Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.render_post(slug).await?;
    Ok(Html(html))
}

#[derive(Deserialize, Debug, Default)]
struct FrontMatter {
    title: Option<String>,
    #[serde(alias = "datePublished")]
    publish_date: Option<String>,
    #[serde(default)]
    slug: Option<String>,
}

#[derive(Clone)]
pub struct Markdown {
    title: Option<String>,
    content: String,
    slugs: Vec<String>,
    publish_date: Option<chrono::NaiveDate>,
}

struct ParsedContent {
    front_matter: Option<FrontMatter>,
    content: String,
}

impl Markdown {
    pub fn from_str(text: &str) -> Self {
        let parsed = Self::parse_front_matter(text);
        match parsed.front_matter {
            Some(front_matter) => Markdown {
                title: front_matter.title,
                content: parsed.content,
                publish_date: front_matter
                    .publish_date
                    .and_then(|s| parse_date_for_sorting(s.as_str())),
                slugs: front_matter.slug.into_iter().collect(),
            },
            None => Markdown {
                title: None,
                content: parsed.content,
                publish_date: None,
                slugs: vec![],
            },
        }
    }

    pub fn contains(&self, slug: String) -> bool {
        self.slugs.contains(&slug)
    }

    // First frontmatter then filename.
    pub fn primary_slug(&self) -> String {
        self.slugs
            .clone()
            .first()
            .expect("Markdown from repo should always contain slug") // todo: make self.title String?
            .to_string()
    }

    fn parse_front_matter(content: &str) -> ParsedContent {
        let matter = Matter::<YAML>::new();
        let result = matter.parse(content);

        let yaml_text = result.matter;
        let content = result.content;

        let front_matter = match serde_yaml::from_str::<FrontMatter>(yaml_text.as_str()) {
            Ok(front_matter) => Some(front_matter),
            Err(e) => {
                eprintln!("Error parsing front matter: {}", e);
                None
            }
        };

        ParsedContent {
            front_matter,
            content,
        }
    }
}

#[derive(serde::Serialize)]
struct BlogPost {
    title: String,
    publish_date: Option<String>,
    slug: String,
}

type ThreadSafeBlogRepository = Arc<dyn BlogRepository + Send + Sync>;
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

    pub async fn list_posts(&self) -> Result<String, StatusCode> {
        let posts = self.get_all_posts()?;

        let mut context = self.build_base_context("/");
        context.insert("posts", &posts);

        self.templates.render("index.html", &context).map_err(|e| {
            eprintln!("Template error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
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

    pub async fn render_page(&self, slug: String) -> Result<String, StatusCode> {
        log::info!("Requested page: {}", &slug);
        let page = self.repo.get_page(&slug).map_err(Self::into)?;
        let markdown = page.ok_or(StatusCode::NOT_FOUND)?;

        let mut context = self.build_base_context(&format!("/p/{}", slug));
        insert_content(&markdown, &mut context);
        insert_title(&markdown, &mut context);

        self.templates.render("page.html", &context).map_err(|e| {
            eprintln!("Template error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    pub async fn render_post(&self, slug: String) -> Result<String, StatusCode> {
        let post = self.repo.find_post_by_slug(&slug).map_err(Self::into)?;
        let markdown = post.ok_or(StatusCode::NOT_FOUND)?;

        let mut context = self.build_base_context(&format!("/{}", slug));
        insert_content(&markdown, &mut context);
        insert_title(&markdown, &mut context);
        insert_published_date(markdown, &mut context);

        self.templates.render("post.html", &context).map_err(|e| {
            eprintln!("Template error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
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

fn parse_date_for_sorting(date_str: &str) -> Option<chrono::NaiveDate> {
    // First try the JavaScript date format (e.g., "Fri Dec 06 2024 12:36:53 GMT+0000")
    if let Ok(datetime) = chrono::DateTime::parse_from_str(
        // Remove the (Coordinated Universal Time) part if present
        date_str.split(" (").next().unwrap_or(date_str),
        "%a %b %d %Y %H:%M:%S GMT%z",
    ) {
        let result = datetime.date_naive();
        return Some(result);
    }

    // Try simple YYYY-MM-DD format
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Some(date);
    }

    // Try "Month Day, Year" format (e.g., "Dec 6, 2024")
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%b %d, %Y") {
        return Some(date);
    }

    // Try with full month name "Month Day, Year" format (e.g., "December 6, 2024")
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        return Some(date);
    }

    None
}

fn format_date_for_post_view(date: chrono::NaiveDate) -> String {
    date.format("%B %d, %Y").to_string()
}

fn format_date_for_posts_overview(date: chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_date_str(date_str: &str) -> String {
        if let Some(date) = parse_date_for_sorting(date_str) {
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
            assert!(parse_date_for_sorting(&formatted).is_some());
        }
    }
}
