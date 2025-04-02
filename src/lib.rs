use axum::{extract::Path, http::StatusCode, response::Html, routing::get, Router};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tera::{Context, Tera};

// Create the app with a default content directory
pub fn create_app() -> Router {
    create_app_with_content_dir(".")
}

// Create the app with a specific content directory
pub fn create_app_with_content_dir<P: Into<PathBuf> + Clone>(content_dir: P) -> Router {
    let blog_handler = Arc::new(BlogPostHandler::new(content_dir.clone()));

    Router::new()
        .route("/health", get(|| async { "I'm ok!" }))
        .route("/", get(index_handler))
        .route("/:slug", get(post_handler))
        .layer(axum::extract::Extension(blog_handler))
}

async fn index_handler(
    blog_handler: axum::extract::Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.list_posts().await?;
    Ok(Html(html))
}

async fn post_handler(
    Path(slug): Path<String>,
    blog_handler: axum::extract::Extension<Arc<BlogPostHandler>>,
) -> Result<Html<String>, StatusCode> {
    let html = blog_handler.render_post(slug).await?;
    Ok(Html(html))
}

#[derive(Deserialize, Debug, Default)]
struct FrontMatter {
    title: Option<String>,
    #[serde(alias = "datePublished")]
    publish_date: Option<String>,
    #[serde(default)]
    slug: Option<String>,
}

#[derive(serde::Serialize)]
struct BlogPost {
    title: String,
    publish_date: Option<String>,
    slug: String,
}

pub struct BlogPostHandler {
    content_dir: PathBuf,
    templates: Tera,
}

impl BlogPostHandler {
    pub fn new<P: Into<PathBuf>>(content_dir: P) -> Self {
        let templates = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Template parsing error(s): {}", e);
                Tera::default()
            }
        };

        Self {
            content_dir: content_dir.into(),
            templates,
        }
    }

    pub async fn list_posts(&self) -> Result<String, StatusCode> {
        let posts = self.get_all_posts()?;

        let mut context = Self::build_base_context("/");
        context.insert("posts", &posts);

        self.templates.render("index.html", &context).map_err(|e| {
            eprintln!("Template error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    fn get_all_posts(&self) -> Result<Vec<BlogPost>, StatusCode> {
        let posts_dir = self.content_dir.join("posts");

        if !posts_dir.exists() {
            return Ok(Vec::new());
        }

        // Temporary struct for sorting
        struct PostWithDate {
            post: BlogPost,
            date: Option<chrono::NaiveDate>,
        }

        let mut posts_with_dates = Vec::new();

        for entry in std::fs::read_dir(posts_dir).map_err(|_| StatusCode::NOT_FOUND)? {
            let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = entry.path();

            if path.extension().map_or(true, |ext| ext != "md") {
                continue;
            }

            let content =
                std::fs::read_to_string(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let front_matter = parse_front_matter(&content).unwrap_or_default();

            // TODO: There is some duplication here with `find_post_by_slug` (e.g. the order of first trying frontmatter for the slug)
            let slug = if let Some(slug) = front_matter.slug {
                slug
            } else if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                file_stem.to_string()
            } else {
                continue; // Skip files without valid slugs
            };

            let post = BlogPost {
                title: front_matter.title.unwrap_or_else(|| "Untitled".to_string()),
                publish_date: front_matter.publish_date.clone(),
                slug,
            };

            let parsed_date = front_matter
                .publish_date
                .as_ref()
                .and_then(|d| parse_date_for_sorting(d));

            posts_with_dates.push(PostWithDate {
                post,
                date: parsed_date,
            });
        }

        fn by_title_alphabetically(a: &PostWithDate, b: &PostWithDate) -> std::cmp::Ordering {
            a.post.title.cmp(&b.post.title)
        }

        fn by_newest_first(
            date_a: &chrono::NaiveDate,
            date_b: &chrono::NaiveDate,
        ) -> std::cmp::Ordering {
            date_b.cmp(date_a)
        }

        // Sort by parsed date
        posts_with_dates.sort_by(|a, b| match (&a.date, &b.date) {
            (Some(date_a), Some(date_b)) => by_newest_first(date_a, date_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => by_title_alphabetically(a, b),
        });

        // Extract sorted posts and format dates for display
        let posts = posts_with_dates
            .into_iter()
            .map(|post_with_date| BlogPost {
                title: post_with_date.post.title,
                publish_date: post_with_date
                    .post
                    .publish_date
                    .as_ref()
                    .map(|date_str| format_date(date_str)),
                slug: post_with_date.post.slug,
            })
            .collect();

        Ok(posts)
    }

    pub async fn render_post(&self, slug: String) -> Result<String, StatusCode> {
        let post_path = self.find_post_by_slug(&slug)?;

        // Try to read the file
        let content = std::fs::read_to_string(post_path).map_err(|_| StatusCode::NOT_FOUND)?;

        // Extract front matter and content using YAML engine
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&content);

        let front_matter = parse_front_matter(&content).unwrap_or_default();

        // Parse the markdown to HTML
        let options = Options::empty();
        let parser = Parser::new_ext(&result.content, options);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);

        let mut context = Self::build_base_context(&format!("/{}", slug));
        if let Some(title) = &front_matter.title {
            context.insert("title", title);
        }
        context.insert("content", &html_content);
        if let Some(date_str) = &front_matter.publish_date {
            // Try to parse the date using multiple possible formats
            let formatted_date = format_date(date_str);
            context.insert("date", &formatted_date);
        }
        //
        // if let Some(tags) = &front_matter.tags {
        // context.insert("tags", tags);
        // }

        // Create the complete HTML with the title
        self.templates.render("post.html", &context).map_err(|e| {
            eprintln!("Template error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }

    fn find_post_by_slug(&self, slug: &str) -> Result<PathBuf, StatusCode> {
        println!("Requested post: {}", &slug);
        let posts_dir = self.content_dir.join("posts");
        for entry in std::fs::read_dir(posts_dir).map_err(|_| StatusCode::NOT_FOUND)? {
            let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "md") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

    fn build_base_context(path: &str) -> Context {
        let mut context = Context::new();

        // Add common values needed by all templates
        let now = chrono::Local::now();
        context.insert("now", &now.to_rfc3339());
        context.insert("current_url", path);

        context
    }
}

fn parse_front_matter(content: &str) -> Option<FrontMatter> {
    // Extract the YAML front matter
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);

    // The matter.parse extracts the YAML as a string
    let yaml_text = result.matter;

    // Try to parse the YAML string into our FrontMatter structure
    match serde_yaml::from_str::<FrontMatter>(yaml_text.as_str()) {
        Ok(front_matter) => Some(front_matter),
        Err(e) => {
            eprintln!("Error parsing front matter: {}", e);
            None
        }
    }
}

fn parse_date_for_sorting(date_str: &str) -> Option<chrono::NaiveDate> {
    // First try the JavaScript date format (e.g., "Fri Dec 06 2024 12:36:53 GMT+0000")
    if let Ok(datetime) = chrono::DateTime::parse_from_str(
        // Remove the (Coordinated Universal Time) part if present
        date_str.split(" (").next().unwrap_or(date_str),
        "%a %b %d %Y %H:%M:%S GMT%z",
    ) {
        let result = datetime.date_naive();
        return Some(result);
    }

    // Try simple YYYY-MM-DD format
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Some(date);
    }

    // Try "Month Day, Year" format (e.g., "Dec 6, 2024")
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%b %d, %Y") {
        return Some(date);
    }

    // Try with full month name "Month Day, Year" format (e.g., "December 6, 2024")
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        return Some(date);
    }

    None
}

fn format_date(date_str: &str) -> String {
    if let Some(date) = parse_date_for_sorting(date_str) {
        // Format the date in a consistent, human-readable format
        return date.format("%B %d, %Y").to_string();
    }

    date_str.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_js_format() {
        // JavaScript date format
        let input = "Fri Dec 06 2024 12:36:53 GMT+0000 (Coordinated Universal Time)";
        let expected = "December 06, 2024";
        assert_eq!(format_date(input), expected);
    }

    #[test]
    fn test_format_date_js_format_without_timezone_name() {
        // JavaScript date format without timezone name in parentheses
        let input = "Fri Dec 06 2024 12:36:53 GMT+0000";
        let expected = "December 06, 2024";
        assert_eq!(format_date(input), expected);
    }

    #[test]
    fn test_format_date_iso_format() {
        // Simple ISO format
        let input = "2024-12-06";
        let expected = "December 06, 2024";
        assert_eq!(format_date(input), expected);
    }

    #[test]
    fn test_format_date_invalid_format() {
        // Invalid format should return the original string
        let input = "Invalid date";
        let expected = "Invalid date";
        assert_eq!(format_date(input), expected);
    }

    #[test]
    fn test_format_date_mixed_format() {
        // Another common format
        let input = "Dec 6, 2024";

        // With our new implementation, this format is now supported
        let expected = "December 06, 2024";
        assert_eq!(format_date(input), expected);
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
            let formatted = format_date(date);
            assert!(parse_date_for_sorting(&formatted).is_some());
        }
    }
}
