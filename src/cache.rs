use std::collections::HashMap;

use axum::http::StatusCode;
use shuttle_axum::axum::response::Html;

use async_trait::async_trait;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::Renderer;

#[derive(Clone)]
struct HtmlCache {
    cache: Arc<RwLock<HashMap<String, Html<String>>>>,
}

impl HtmlCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get(&self, key: &str) -> Option<Html<String>> {
        self.cache.read().await.get(key).cloned()
    }

    async fn insert(&self, key: String, html: Html<String>) {
        self.cache.write().await.insert(key, html);
    }
}

#[derive(Clone)]
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

    pub async fn preload_posts(&self, slugs: Vec<String>) -> Result<(), StatusCode> {
        // Pre-render all posts and cache them
        for slug in slugs {
            let cache_key = format!("post:{}", slug);

            // Only preload if not already cached
            if self.cache.get(&cache_key).await.is_none() {
                let rendered_html = self.renderer.post_for(slug).await?;
                self.cache.insert(cache_key, rendered_html).await;
            }
        }
        Ok(())
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
