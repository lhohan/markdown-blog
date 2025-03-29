use axum::{extract::Path, http::StatusCode, response::Html, routing::get, Router};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

// Create the app with a default content directory
pub fn create_app() -> Router {
    create_app_with_content_dir(".")
}

// Create the app with a specific content directory
pub fn create_app_with_content_dir<P: Into<PathBuf> + Clone>(content_dir: P) -> Router {
    let blog_handler = Arc::new(BlogPostHandler::new(content_dir.clone()));

    Router::new()
        .route("/health", get(|| async { "I'm ok!" }))
        .route("/:slug", get(post_handler))
        .layer(axum::extract::Extension(blog_handler))
}

// Handler function for the post route
async fn post_handler(
    Path(slug): Path<String>,
    blog_handler: axum::extract::Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.render_post(slug).await?;
    Ok(Html(html))
}

#[derive(Deserialize, Debug)]
struct FrontMatter {
    title: String,
    #[serde(alias = "datePublished")]
    publish_date: Option<String>,
    #[serde(default)]
    slug: Option<String>,
}

pub struct BlogPostHandler {
    content_dir: PathBuf,
}

impl BlogPostHandler {
    pub fn new<P: Into<PathBuf>>(content_dir: P) -> Self {
        Self {
            content_dir: content_dir.into(),
        }
    }

    // Render a post from its slug
    pub async fn render_post(&self, slug: String) -> Result<String, StatusCode> {
        let post_path = self.find_post_by_slug(&slug)?;

        // Try to read the file
        let content = std::fs::read_to_string(post_path).map_err(|_| StatusCode::NOT_FOUND)?;

        // Extract front matter and content using YAML engine
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&content);

        // Parse the front matter
        let front_matter = parse_front_matter(&content).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Parse the markdown to HTML
        let options = Options::empty();
        let parser = Parser::new_ext(&result.content, options);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);

        // Create the complete HTML with the title
        let html_output = format!(
            r#"<!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{}</title>
        <style>
            body {{
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                line-height: 1.6;
                color: #333;
                max-width: 800px;
                margin: 0 auto;
                padding: 1rem;
            }}
            h1 {{
                color: #222;
            }}
        </style>
    </head>
    <body>
        <h1>{}</h1>
        {}
    </body>
    </html>"#,
            front_matter.title, front_matter.title, html_content
        );

        Ok(html_output)
    }

    fn find_post_by_slug(&self, slug: &str) -> Result<PathBuf, StatusCode> {
        dbg!(&slug);
        let posts_dir = self.content_dir.join("posts");
        for entry in std::fs::read_dir(posts_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
            let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "md") {
                let content = std::fs::read_to_string(&path).map_err(|_| StatusCode::NOT_FOUND)?;

                // Try to parse front matter
                if let Some(front_matter) = parse_front_matter(&content) {
                    // Check if front matter has a slug that matches
                    if let Some(front_matter_slug) = &front_matter.slug {
                        if front_matter_slug == slug {
                            return Ok(path);
                        }
                    }
                }

                // If no match by front matter slug, check if the filename (without extension) matches
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if file_stem == slug {
                        return Ok(path);
                    }
                }
            }
        }
        Err(StatusCode::NOT_FOUND)
    }
}

fn parse_front_matter(content: &str) -> Option<FrontMatter> {
    // Extract the YAML front matter
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);

    // The matter.parse extracts the YAML as a string
    let yaml_text = result.matter;

    // Try to parse the YAML string into our FrontMatter structure
    serde_yaml::from_str::<FrontMatter>(yaml_text.as_str()).ok()
}
