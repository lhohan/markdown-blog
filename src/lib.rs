mod blog_repository;
mod config;
use blog_repository::{BlogRepository, FileSystemBlogRepository, RepositoryError};
pub use config::BlogConfig;

use axum::{
    extract::Path,
    http::StatusCode,
    response::Html,
    routing::{get, get_service},
    Router,
};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

pub fn create_app_with_defaults() -> Router {
    create_app_with_content_dir(".")
}

pub fn create_app_with_content_dir<P: Into<PathBuf> + Clone>(content_dir: P) -> Router {
    let config_path = content_dir.clone().into().join("blog_config.yaml");
    let config = BlogConfig::from_file_or_default(config_path);
    create_app(content_dir, config)
}

fn create_app<P: Into<PathBuf> + Clone>(content_dir: P, config: BlogConfig) -> Router {
    let repo = FileSystemBlogRepository::new(content_dir.clone().into());
    let blog_handler = Arc::new(BlogPostHandler::new(config, repo));

    let static_service = get_service(ServeDir::new("static")).handle_error(|error| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error serving static file: {}", error),
        )
    });

    Router::new()
        .route("/health", get(|| async { "I'm ok!" }))
        .route("/", get(index_handler))
        .route("/p/:slug", get(page_handler))
        .route("/:slug", get(post_handler))
        .nest_service("/static", static_service)
        .layer(axum::extract::Extension(blog_handler))
}

async fn index_handler(
    blog_handler: axum::extract::Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.list_posts().await?;
    Ok(Html(html))
}

async fn page_handler(
    Path(slug): Path<String>,
    blog_handler: axum::extract::Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.render_page(slug).await?;
    Ok(Html(html))
}

async fn post_handler(
    Path(slug): Path<String>,
    blog_handler: axum::extract::Extension<Arc<BlogPostHandler>>,
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

pub struct Markdown {
    title: Option<String>,
    content: String,
    repo_slug: Option<String>,
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
                repo_slug: front_matter.slug,
            },
            None => Markdown {
                title: None,
                content: parsed.content,
                publish_date: None,
                repo_slug: None,
            },
        }
    }

    pub fn slug(&self) -> String {
        self.repo_slug
            .clone()
            .expect("Markdown from repo should always contain slug") // todo: make self.title String?
    }

    fn parse_front_matter(content: &str) -> ParsedContent {
        let matter = Matter::<YAML>::new();
        let result = matter.parse(content);

        let yaml_text = result.matter;
        let content = result.content;

        let front_matter = serde_yaml::from_str::<FrontMatter>(yaml_text.as_str())
            .map_err(|e| {
                eprintln!("Error parsing front matter: {}", e);
                e
            })
            .ok();

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
    pub fn new(config: BlogConfig, blog_repo: impl BlogRepository + Send + Sync + 'static) -> Self {
        let templates = match Tera::new("templates/**/*.html") {
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
                publish_date: markdown.publish_date.map(format_date),
                slug: markdown.slug(),
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
        let formatted_date = format_date(*date_str);
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

fn format_date(date: chrono::NaiveDate) -> String {
    date.format("%B %d, %Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_date_str(date_str: &str) -> String {
        if let Some(date) = parse_date_for_sorting(date_str) {
            return format_date(date);
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
