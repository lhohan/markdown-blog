mod blog_repository;
mod cache;
mod config;
mod model;
mod renderer;
use blog_repository::FileSystemBlogRepository;
pub use config::BlogConfig;
use model::Markdown;
use renderer::{BlogPostHandler, Renderer};

use shuttle_axum::axum::{
    extract::Path,
    http::StatusCode,
    response::Html,
    routing::{get, get_service},
    Extension, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::format_date_for_post_view;

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
