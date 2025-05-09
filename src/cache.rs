use std::collections::HashMap;
use std::time::Instant;

use crate::{BlogRepository, Markdown, RepositoryError};

pub struct Cached<R, T> {
    inner: R,
    cache: Cache<T>,
}

struct Cache<T> {
    items: Vec<T>,
    slug_to_item: HashMap<String, T>,
    last_refresh: Instant,
}

impl<T: Clone> Cache<T> {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            slug_to_item: HashMap::new(),
            last_refresh: Instant::now(),
        }
    }

    fn get(&self, slug: String) -> Option<&T> {
        self.slug_to_item.get(&slug)
    }
}

impl<R: BlogRepository> Cached<R, Markdown> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            cache: Cache::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<(), RepositoryError> {
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

impl<R: BlogRepository> BlogRepository for Cached<R, Markdown> {
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
