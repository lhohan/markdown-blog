mod blog_repository;
mod cache;
mod config;
mod model;
mod renderer;
use blog_repository::FileSystemBlogRepository;
pub use config::BlogConfig;
pub use directories::{BlogDir, ContentDir};
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

fn create_app(content_dir: ContentDir, blog_dir: &BlogDir, config: BlogConfig) -> Router {
    let renderer = {
        let repo = create_repo(content_dir.dir());
        create_renderer(blog_dir, config, repo)
    };

    Router::new()
        .route("/health", get(|| async { "I'm ok!" }))
        .route("/", get(index_handler))
        .route("/p/{slug}", get(page_handler))
        .route("/{slug}", get(post_handler))
        .nest_service("/static", static_handler(blog_dir))
        .layer(axum::extract::Extension(renderer))
}

fn static_handler(blog_dir: &BlogDir) -> shuttle_axum::axum::routing::MethodRouter {
    let statics = blog_dir.static_dir();
    let static_handler = get_service(ServeDir::new(statics)).handle_error(|error| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error serving static file: {}", error),
        )
    });
    static_handler
}

fn create_repo<P: Into<PathBuf> + Clone>(content_dir: P) -> FileSystemBlogRepository {
    FileSystemBlogRepository::new(content_dir.clone().into())
}

fn create_renderer(
    blog_dir: &BlogDir,
    config: BlogConfig,
    repo: FileSystemBlogRepository,
) -> Arc<dyn Renderer + Send + Sync + 'static> {
    let blog_handler = BlogPostHandler::new(config, repo, blog_dir);

    let slugs = match blog_handler.get_all_post_slugs() {
        Ok(slugs) => slugs,
        Err(e) => {
            eprintln!(
                "Warning: Could not get post slugs for cache preloading: {:?}",
                e
            );
            Vec::new()
        }
    };

    let cached_renderer = CachedRenderer::new(Arc::new(blog_handler));
    let preloaded_renderer = cached_renderer.clone();
    tokio::spawn(async move {
        if let Err(e) = preloaded_renderer.preload_posts(slugs).await {
            eprintln!("Warning: Cache preloading failed: {:?}", e);
        }
    });

    let shared_renderer: Arc<dyn Renderer + Send + Sync> = Arc::new(cached_renderer);
    shared_renderer
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

mod directories {
    use std::path::PathBuf;

    // Files part of this blog
    pub struct BlogDir(pub PathBuf);

    // Blog content
    pub struct ContentDir(pub PathBuf);

    impl BlogDir {
        pub fn dir(&self) -> PathBuf {
            self.0.clone()
        }
        pub fn templates_dir(&self) -> PathBuf {
            self.dir().join("templates")
        }
        pub fn static_dir(&self) -> PathBuf {
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
}
