use std::collections::HashMap;
use std::time::Instant;

use crate::{BlogRepository, Markdown, RepositoryError};

pub struct CachedBlogRepository<R> {
    inner: R,
    cache: Cache,
}

struct Cache {
    posts: Vec<Markdown>,
    slug_map: HashMap<String, usize>,
    last_refresh: Instant,
}

impl Cache {
    fn new() -> Self {
        Self {
            posts: Vec::new(),
            slug_map: HashMap::new(),
            last_refresh: Instant::now(),
        }
    }
}

impl<R: BlogRepository> CachedBlogRepository<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            cache: Cache::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<(), RepositoryError> {
        let posts = self.inner.get_all_posts()?;
        let mut slug_map = HashMap::new();

        for (idx, post) in posts.iter().enumerate() {
            for slug in &post.slugs {
                slug_map.insert(slug.clone(), idx);
            }
        }

        self.cache.posts = posts;
        self.cache.slug_map = slug_map;
        self.cache.last_refresh = Instant::now();

        Ok(())
    }
}

impl<R: BlogRepository> BlogRepository for CachedBlogRepository<R> {
    fn get_all_posts(&self) -> Result<Vec<Markdown>, RepositoryError> {
        Ok(self.cache.posts.clone())
    }

    fn find_post_by_slug(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError> {
        Ok(self
            .cache
            .slug_map
            .get(slug)
            .map(|&idx| self.cache.posts[idx].clone()))
    }

    fn get_page(&self, slug: &str) -> Result<Option<Markdown>, RepositoryError> {
        // Pages are not cached yet
        self.inner.get_page(slug)
    }
}
