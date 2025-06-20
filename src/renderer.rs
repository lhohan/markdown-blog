use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Html;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, html};
use std::sync::Arc;

use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use tera::{Context, Tera};

use crate::BlogDir;
use crate::blog_repository::{BlogRepository, RepositoryError};
use crate::config::BlogConfig;
use crate::model::{BlogPost, Markdown, format_date_for_post_view, format_date_for_posts_overview};

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
    syntax_set: SyntaxSet,
    theme: Theme,
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

        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults().themes["base16-ocean.dark"].clone();

        Self {
            repo,
            templates,
            config,
            syntax_set,
            theme,
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
        self.insert_content(&markdown, &mut context);
        Self::insert_title(&markdown, &mut context);

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
        self.insert_content(&markdown, &mut context);
        Self::insert_title(&markdown, &mut context);
        Self::insert_published_date(markdown, &mut context);

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

    fn insert_content(&self, markdown: &Markdown, context: &mut Context) {
        let html_content = self.parse_to_html(markdown);
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

    fn parse_to_html(&self, markdown: &Markdown) -> String {
        let mut options = Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        options.insert(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(&markdown.content, options);
        let mut events = Vec::new();
        let mut code_block: Option<(String, String)> = None;

        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        CodeBlockKind::Fenced(lang) => lang.into_string(),
                        CodeBlockKind::Indented => "text".to_string(),
                    };
                    code_block = Some((lang, String::new()));
                }
                Event::Text(text) => {
                    if let Some((_, content)) = &mut code_block {
                        content.push_str(&text);
                    } else {
                        events.push(Event::Text(text));
                    }
                }
                Event::End(Tag::CodeBlock(_)) => {
                    if let Some((lang, content)) = code_block.take() {
                        let syntax = self
                            .syntax_set
                            .find_syntax_by_token(&lang)
                            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

                        let highlighted_code = highlighted_html_for_string(
                            &content,
                            &self.syntax_set,
                            syntax,
                            &self.theme,
                        )
                        .unwrap_or_else(|_| content.clone());
                        let html = format!("<code>{}</code>", highlighted_code);
                        events.push(Event::Html(html.into()));
                    }
                }
                e => {
                    events.push(e);
                }
            }
        }

        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());
        html_output
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
