use std::collections::HashMap;
use std::time::Instant;

use axum::http::StatusCode;
use shuttle_axum::axum::response::Html;

use async_trait::async_trait;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{BlogRepository, Markdown, Renderer, RepositoryError};

pub struct Cached<R> {
    inner: R,
    cache: DataCache,
}

struct HtmlCache {
    cache: RwLock<HashMap<String, Html<String>>>,
}

impl HtmlCache {
    fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    async fn get(&self, key: &str) -> Option<Html<String>> {
        self.cache.read().await.get(key).cloned()
    }

    async fn insert(&self, key: String, html: Html<String>) {
        self.cache.write().await.insert(key, html);
    }
}

pub struct CachedRenderer {
    renderer: Arc<dyn Renderer + Send + Sync>,
    cache: HtmlCache,
}

impl CachedRenderer {
    pub fn new(renderer: Arc<dyn Renderer + Send + Sync>) -> Self {
        Self {
            renderer,
            cache: HtmlCache::new(),
        }
    }
}

struct DataCache {
    items: Vec<Markdown>,
    slug_to_item: HashMap<String, Markdown>,
    last_refresh: Instant,
}

impl DataCache {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            slug_to_item: HashMap::new(),
            last_refresh: Instant::now(),
        }
    }

    fn get(&self, slug: String) -> Option<&Markdown> {
        self.slug_to_item.get(&slug)
    }
}

impl<R: BlogRepository> Cached<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            cache: DataCache::new(),
        }
    }

    pub fn preload_all(&mut self) -> Result<(), RepositoryError> {
        let posts = self.inner.get_all_posts()?;
        let mut slug_map = HashMap::new();

        for post in posts.iter() {
            for slug in &post.slugs {
                slug_map.insert(slug.clone(), post.clone());
            }
        }

        self.cache.items = posts;
        self.cache.slug_to_item = slug_map;
        self.cache.last_refresh = Instant::now();

        Ok(())
    }
}

impl<R: BlogRepository> BlogRepository for Cached<R> {
    fn get_all_posts(&self) -> Result<Vec<Markdown>, RepositoryError> {
        Ok(self.cache.items.clone())
    }

    fn find_post_by_slug(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError> {
        Ok(self.cache.get(slug.to_string()).cloned())
    }

    fn get_page(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError> {
        // Pages are not cached yet
        self.inner.get_page(slug)
    }
}

#[async_trait]
impl Renderer for CachedRenderer {
    async fn post_for(&self, slug: String) -> Result<Html<String>, StatusCode> {
        let cache_key = format!("post:{}", slug);
        if let Some(cached_html) = self.cache.get(&cache_key).await {
            return Ok(cached_html);
        }

        let rendered_html = self.renderer.post_for(slug).await?;
        self.cache.insert(cache_key, rendered_html.clone()).await;
        Ok(rendered_html)
    }

    async fn page_for(&self, slug: String) -> Result<Html<String>, StatusCode> {
        let cache_key = format!("page:{}", slug);
        if let Some(cached_html) = self.cache.get(&cache_key).await {
            return Ok(cached_html);
        }

        let rendered_html = self.renderer.page_for(slug).await?;
        self.cache.insert(cache_key, rendered_html.clone()).await;
        Ok(rendered_html)
    }

    async fn posts(&self) -> Result<Html<String>, StatusCode> {
        let cache_key = "posts_index".to_string();
        if let Some(cached_html) = self.cache.get(&cache_key).await {
            return Ok(cached_html);
        }

        let rendered_html = self.renderer.posts().await?;
        self.cache.insert(cache_key, rendered_html.clone()).await;
        Ok(rendered_html)
    }
}
