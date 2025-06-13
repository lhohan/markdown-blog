use async_trait::async_trait;
use axum::http::StatusCode;
use pulldown_cmark::{html, Options, Parser};
use shuttle_axum::axum::response::Html;
use std::sync::Arc;
use tera::{Context, Tera};

use crate::blog_repository::{BlogRepository, RepositoryError};
use crate::config::BlogConfig;
use crate::model::{format_date_for_post_view, format_date_for_posts_overview, BlogPost, Markdown};
use crate::BlogDir;

pub type ThreadSafeBlogRepository = Arc<dyn BlogRepository + Send + Sync>;

#[async_trait]
pub trait Renderer {
    async fn post_for(&self, slug: String) -> Result<Html<String>, StatusCode>;
    async fn page_for(&self, slug: String) -> Result<Html<String>, StatusCode>;
    async fn posts(&self) -> Result<Html<String>, StatusCode>;
}

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

    pub fn get_all_post_slugs(&self) -> Result<Vec<String>, StatusCode> {
        let markdowns = self.repo.get_all_posts().map_err(Self::into)?;
        let slugs = markdowns
            .into_iter()
            .map(|markdown| markdown.primary_slug())
            .collect();
        Ok(slugs)
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
            RepositoryError::NotFound => StatusCode::NOT_FOUND,
            RepositoryError::UnexpectedError => StatusCode::INTERNAL_SERVER_ERROR,
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
