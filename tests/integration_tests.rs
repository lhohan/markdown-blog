use rstest::rstest;
use specification_support::IntoAssertion;

use crate::specification_support::BlogServer;

#[tokio::test]
async fn health_endpoint_should_return_200() {
    BlogServer::new()
        .start()
        .await
        .get("/health")
        .await
        .expect()
        .await
        .http_status_code(200);
}

#[tokio::test]
async fn post_should_be_served_from_file_name() {
    let post_content = r#"---
title: Test Post
datePublished: 2023-01-01
---
# Hello World

This is a test blog post.
"#;
    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/test-post")
        .await
        .expect()
        .await
        .http_status_code(200)
        .body_contains("<h1>Hello World</h1>")
        .body_contains("This is a test blog post.");
}

#[tokio::test]
async fn post_should_be_served_from_slug_from_frontmatter() {
    let post_content = r#"---
title: Test Post
datePublished: 2023-01-01
slug: hello
---
This is a test blog post.
"#;

    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/hello")
        .await
        .expect()
        .await
        .http_status_code(200);
}

#[tokio::test]
async fn post_should_contain_simple_markdown_content() {
    let post_content = r#"---
title: Test Post
datePublished: 2023-01-01
slug: hello
---
This is a test blog post.
"#;

    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/hello")
        .await
        .expect()
        .await
        .body_contains("This is a test blog post.");
}

#[tokio::test]
async fn post_should_contain_title_from_frontmatter() {
    let post_content = r#"---
title: Hello World
datePublished: 2023-01-01
slug: hello
---
This is a test blog post.
"#;

    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/hello")
        .await
        .expect()
        .await
        .body_contains("Hello World</h1>");
}

#[tokio::test]
async fn post_should_contain_date_published_from_frontmatter() {
    let post_content = r#"---
title: Hello World
datePublished: 2023-01-01
slug: hello
---
This is a test blog post.
"#;

    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/hello")
        .await
        .expect()
        .await
        .body_contains("Published on January 01, 2023");
}

#[tokio::test]
async fn post_should_not_be_served_from_slug_front_matter_misses_closing_delimiter() {
    let post_content = r#"---
title: Test Post
datePublished: 2023-01-01
slug: hello

# Hello World"#;

    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/hello")
        .await
        .expect()
        .await
        .http_status_code(404);
}

#[tokio::test]
async fn post_should_be_served_from_filename_when_there_is_no_front_matter() {
    let post_content = r#"# Raw Markdown Post

This is a blog post without any front matter.
It should still be displayed properly."#;

    BlogServer::with_file("posts/no-front-matter.md", post_content)
        .start()
        .await
        .get("/no-front-matter")
        .await
        .expect()
        .await
        .http_status_code(200)
        .body_contains("<h1>Raw Markdown Post</h1>")
        .body_contains("This is a blog post without any front matter.");
}

#[tokio::test]
async fn non_existent_slug_should_return_404() {
    BlogServer::new()
        .start()
        .await
        .get("/non-existent-post")
        .await
        .expect()
        .await
        .http_status_code(404);
}

#[tokio::test]
async fn index_should_show_posts() {
    let post1 = r#"---
title: First Post
datePublished: 2023-01-02
---
# First Post Content
"#;

    let post2 = r#"---
title: Second Post
datePublished: 2023-01-01
---
# Second Post Content
"#;

    BlogServer::new()
        .add_file("posts/first-post.md", post1)
        .add_file("posts/second-post.md", post2)
        .start()
        .await
        .get("/")
        .await
        .expect()
        .await
        .http_status_code(200)
        .body_contains("First Post")
        .body_contains("Second Post");
}

#[tokio::test]
async fn index_should_show_no_posts_message_when_no_posts() {
    BlogServer::new()
        .start()
        .await
        .get("/")
        .await
        .expect()
        .await
        .http_status_code(200)
        .body_contains("No posts yet");
}

#[tokio::test]
async fn index_should_sort_posts_by_date_newest_first() {
    // Create posts with various date formats spanning different years
    let posts = [
        (
            "posts/oldest-post.md",
            r#"---
title: Oldest Post
datePublished: 2020-01-15
---
# Oldest Post Content
"#,
        ),
        (
            "posts/middle-post-js-date.md",
            r#"---
title: Middle Post (JS Date)
datePublished: Wed Jun 15 2022 10:30:00 GMT+0000
---
# Middle Post Content with JS Date Format
"#,
        ),
        (
            "posts/newer-post-short-format.md",
            r#"---
title: Newer Post (Short Format)
datePublished: Mar 20, 2023
---
# Newer Post with Short Date Format
"#,
        ),
        (
            "posts/newest-post-full-format.md",
            r#"---
title: Newest Post (Full Format)
datePublished: December 25, 2024
---
# Newest Post with Full Month Format
"#,
        ),
        (
            "posts/undated-post.md",
            r#"---
title: Undated Post Z (should be last, alphabetically)
---
# Undated Post Content
"#,
        ),
        (
            "posts/another-undated-post.md",
            r#"---
title: Another Undated Post A (should be first among undated, alphabetically)
---
# Another Undated Post Content
"#,
        ),
    ];

    // Set up the server with all posts
    let mut server = BlogServer::new();
    for (path, content) in posts {
        server = server.add_file(path, content);
    }

    // Get the index page
    let response = server.start().await.get("/").await;

    // Check the response
    response
        .expect()
        .await
        .http_status_code(200)
        // First verify all posts are present
        .body_contains("Oldest Post")
        .body_contains("Middle Post (JS Date)")
        .body_contains("Newer Post (Short Format)")
        .body_contains("Newest Post (Full Format)")
        .body_contains("Undated Post Z")
        .body_contains("Another Undated Post A")
        .contains_in_order(&[
            "Newest Post (Full Format)",
            "Newer Post (Short Format)",
            "Middle Post (JS Date)",
            "Oldest Post",
        ])
        // Verify undated posts come after dated posts, sorted alphabetically
        .contains_in_order(&[
            "Oldest Post",            // Last dated post
            "Another Undated Post A", // First undated post (alphabetically)
            "Undated Post Z",         // Second undated post (alphabetically)
        ]);
}

#[tokio::test]
#[rstest]
async fn all_should_show_custom_title_when_configured(
    #[values(
        TestSetupShouldShowCustomTitleWhenConfigured::index(),
        TestSetupShouldShowCustomTitleWhenConfigured::post(),
        TestSetupShouldShowCustomTitleWhenConfigured::page()
    )]
    setup: TestSetupShouldShowCustomTitleWhenConfigured,
) {
    let config = r#"
site_title: "My Custom Blog Title"
site_description: "A custom blog description"
"#;

    let setup_server = setup.server;
    let slug_matching_content_on_server = setup.slug.as_str();

    setup_server
        .with_config(config)
        .start()
        .await
        .get(slug_matching_content_on_server)
        .await
        .expect()
        .await
        .http_status_code(200)
        .body_contains("My Custom Blog Title")
        .not_contains("Your Blog"); // the default site title
}

struct TestSetupShouldShowCustomTitleWhenConfigured {
    server: BlogServer,
    slug: String,
}
impl TestSetupShouldShowCustomTitleWhenConfigured {
    fn index() -> Self {
        Self {
            server: BlogServer::new(),
            slug: "/".to_string(),
        }
    }
    fn post() -> Self {
        Self {
            server: BlogServer::with_file("posts/post.md", "# Post Title"),
            slug: "/post".to_string(),
        }
    }
    fn page() -> Self {
        Self {
            server: BlogServer::with_file("pages/page.md", "# Page Title"),
            slug: "/p/page".to_string(),
        }
    }
}

#[tokio::test]
async fn page_should_be_served() {
    let page_content = r#"# Raw Page Title

This is a page without any front matter.
It should still be displayed properly."#;

    BlogServer::new()
        .add_file("pages/about.md", page_content)
        .start()
        .await
        .get("/p/about")
        .await
        .expect()
        .await
        .http_status_code(200)
        .body_contains("<h1>Raw Page Title</h1>");
}

mod specification_support {
    use axum::Router;
    use axum::serve;
    use blog_engine::BlogDir;
    use blog_engine::ContentDir;
    use blog_engine::create_app_with_dirs;
    use std::fs;
    use std::net::SocketAddr;
    use tempfile::TempDir;

    pub struct BlogServer {
        content_on_server: Vec<FileOnServer>,
        config: Option<String>,
    }

    impl BlogServer {
        pub fn new() -> Self {
            BlogServer {
                content_on_server: Vec::new(),
                config: None,
            }
        }

        pub fn with_file(target_path: &str, content: &str) -> Self {
            BlogServer {
                content_on_server: vec![FileOnServer {
                    target_path: target_path.to_string(),
                    content: content.to_string(),
                }],
                config: None,
            }
        }

        pub fn add_file(self, target_path: &str, content: &str) -> Self {
            let new_file = FileOnServer {
                target_path: target_path.to_string(),
                content: content.to_string(),
            };

            let mut content_on_server = self.content_on_server;
            content_on_server.push(new_file);

            BlogServer {
                content_on_server,
                ..self
            }
        }

        pub fn with_config(mut self, config_yaml: &str) -> Self {
            self.config = Some(config_yaml.to_string());
            self
        }

        pub async fn start(self) -> RunningServer {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().to_owned();

            for file_on_server in self.content_on_server {
                let file_path = &file_on_server.target_path;
                let file = temp_path.join(file_path);

                if let Some(parent) = file.parent() {
                    fs::create_dir_all(parent).unwrap();
                }

                fs::write(&file, &file_on_server.content).unwrap();
            }

            let blog_dir = BlogDir(".".into());
            let content_dir = ContentDir(temp_path.clone());
            if let Some(config_content) = self.config {
                fs::write(&content_dir.config_file(), &config_content).unwrap();
            }

            let app = create_app_with_dirs(temp_path, blog_dir.dir());
            let (server_addr, shutdown_tx, server_handle) = start_test_server(app).await;

            // Keep temp_dir in the RunningServer so it lives as long as the server
            RunningServer::new(server_addr, shutdown_tx, server_handle, temp_dir)
        }
    }

    pub struct RunningServer {
        server_addr: SocketAddr,
        shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
        server_handle: Option<tokio::task::JoinHandle<()>>,
        _temp_dir: TempDir, // Keep the directory alive as long as the server
    }

    impl RunningServer {
        pub fn new(
            server_addr: SocketAddr,
            shutdown_tx: tokio::sync::oneshot::Sender<()>,
            server_handle: tokio::task::JoinHandle<()>,
            temp_dir: TempDir,
        ) -> Self {
            RunningServer {
                server_addr,
                shutdown_tx: Some(shutdown_tx),
                server_handle: Some(server_handle),
                _temp_dir: temp_dir,
            }
        }

        pub async fn get(&self, path: &str) -> reqwest::Response {
            let url = format!("http://{}{}", self.server_addr, path);
            reqwest::Client::new().get(url).send().await.unwrap()
        }

        fn shutdown_sync(&mut self) {
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(());
            }

            if let Some(handle) = self.server_handle.take() {
                handle.abort();
            }
        }
    }

    impl Drop for RunningServer {
        fn drop(&mut self) {
            self.shutdown_sync();
        }
    }

    async fn start_test_server(
        app: Router,
    ) -> (
        SocketAddr,
        tokio::sync::oneshot::Sender<()>,
        tokio::task::JoinHandle<()>,
    ) {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let shutdown_future = async {
            let _ = shutdown_rx.await;
        };

        let server_handle = tokio::spawn(async move {
            serve(listener, app)
                .with_graceful_shutdown(shutdown_future)
                .await
                .unwrap();
        });

        (server_addr, shutdown_tx, server_handle)
    }

    pub struct ResponseExpectations {
        body: String,
        status_code: u16,
        expectations: Vec<Box<dyn FnOnce(&str) + Send>>,
        verified: bool,
    }

    impl ResponseExpectations {
        pub async fn new(response: reqwest::Response) -> Self {
            let status_code = response.status().as_u16();
            let body = response.text().await.expect("Failed to read response body");
            Self {
                body,
                status_code,
                expectations: Vec::new(),
                verified: false,
            }
        }

        pub fn http_status_code(mut self, expected: u16) -> Self {
            let status = self.status_code;
            self.expectations.push(Box::new(move |_| {
                assert_eq!(
                    status, expected,
                    "Expected status code {}, got {}",
                    expected, status
                );
            }));
            self
        }

        pub fn body_contains(mut self, substring: &str) -> Self {
            let substring = substring.to_string();
            self.expectations.push(Box::new(move |body| {
                assert!(
                    body.contains(&substring),
                    "Response body does not contain '{}'. Full body:\n{}",
                    substring,
                    body
                );
            }));
            self
        }

        // TODO: Review this implementation. Seems to work but copy-pasted from Claude.
        pub fn contains_in_order(mut self, substrings: &[&str]) -> Self {
            // Clone the substrings for the closure
            let substrings: Vec<String> = substrings.iter().map(|s| s.to_string()).collect();

            self.expectations.push(Box::new(move |body| {
                    let mut last_pos = 0;

                    for (i, substring) in substrings.iter().enumerate() {
                        match body[last_pos..].find(substring) {
                            Some(pos) => {
                                // Update position for next search
                                last_pos += pos + substring.len();
                            }
                            None => {
                                assert!(
                                    false,
                                    "Expected to find '{}' after position {}. Items should appear in order: {:?}. Full body:\n{}",
                                    substring,
                                    last_pos,
                                    substrings,
                                    body
                                );
                            }
                        }

                        // For all but the last item, ensure the next item comes after this one
                        if i < substrings.len() - 1 {
                            let next = &substrings[i + 1];
                            match body[last_pos..].find(next) {
                                Some(_) => { /* This is good, the next item appears after this one */ }
                                None => {
                                    assert!(
                                        false,
                                        "Expected to find '{}' after '{}', but it was not found. Full body:\n{}",
                                        next,
                                        substring,
                                        body
                                    );
                                }
                            }
                        }
                    }
                }));

            self
        }

        pub fn not_contains(mut self, substring: &str) -> Self {
            let substring = substring.to_string();
            self.expectations.push(Box::new(move |body| {
                assert!(
                    !body.contains(&substring),
                    "Response body should not contain '{}'. Full body:\n{}",
                    substring,
                    body
                );
            }));
            self
        }
    }

    impl Drop for ResponseExpectations {
        fn drop(&mut self) {
            if !self.verified {
                let expectations = std::mem::take(&mut self.expectations);
                for expectation in expectations {
                    expectation(&self.body);
                }
            }
        }
    }
    pub trait IntoAssertion {
        async fn expect(self) -> ResponseExpectations;
    }

    impl IntoAssertion for reqwest::Response {
        async fn expect(self) -> ResponseExpectations {
            ResponseExpectations::new(self).await
        }
    }

    struct FileOnServer {
        content: String,
        target_path: String,
    }
}
