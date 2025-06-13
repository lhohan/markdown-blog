use std::collections::HashMap;

use axum::http::StatusCode;
use shuttle_axum::axum::response::Html;

use async_trait::async_trait;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::Renderer;

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
