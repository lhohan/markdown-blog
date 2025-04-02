use specification_support::IntoAssertion;

use crate::specification_support::BlogServer;

#[tokio::test]
async fn health_endpoint_returns_200() {
    BlogServer::empty()
        .start()
        .await
        .get("/health")
        .await
        .expect()
        .status(200)
        .contains("I'm ok!")
        .verify()
        .await;
}

#[tokio::test]
async fn can_serve_single_blog_post() {
    let post_content = r#"---
title: Test Post
datePublished: 2023-01-01
---
# Hello World

This is a test blog post.
"#;
    BlogServer::empty()
        .add_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/test-post")
        .await
        .expect()
        .status(200)
        .contains("<h1>Hello World</h1>")
        .contains("This is a test blog post.")
        .verify()
        .await;
}

#[tokio::test]
async fn can_serve_single_blog_post_with_slug_from_frontmatter() {
    let post_content = r#"---
title: Test Post
datePublished: 2023-01-01
slug: abc123
---
# Hello World

This is a test blog post.
"#;

    BlogServer::with_file("posts/test-post.md", post_content)
        .start()
        .await
        .get("/abc123")
        .await
        .expect()
        .status(200)
        .contains("<h1>Hello World</h1>")
        .contains("This is a test blog post.")
        .verify()
        .await;
}

#[tokio::test]
async fn can_serve_blog_post_without_front_matter() {
    let post_content_without_front_matter = r#"# Raw Markdown Post

This is a blog post without any front matter.
It should still be displayed properly."#;

    BlogServer::with_file(
        "posts/no-front-matter.md",
        post_content_without_front_matter,
    )
    .start()
    .await
    .get("/no-front-matter")
    .await
    .expect()
    .status(200)
    .contains("<h1>Raw Markdown Post</h1>")
    .contains("This is a blog post without any front matter.")
    .verify()
    .await;
}

#[tokio::test]
async fn returns_404_on_nonexistent_posts() {
    BlogServer::empty()
        .start()
        .await
        .get("/non-existent-post")
        .await
        .expect()
        .status(404)
        .verify()
        .await;
}

#[tokio::test]
async fn returns_404_on_nonexistent_post() {
    BlogServer::empty()
        .add_file("posts/abc123", "content")
        .start()
        .await
        .get("/non-existent-post")
        .await
        .expect()
        .status(404)
        .verify()
        .await;
}

#[tokio::test]
async fn index_page_shows_blog_posts() {
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

    BlogServer::empty()
        .add_file("posts/first-post.md", post1)
        .add_file("posts/second-post.md", post2)
        .start()
        .await
        .get("/")
        .await
        .expect()
        .status(200)
        .contains("First Post")
        .contains("Second Post")
        .verify()
        .await;
}

#[tokio::test]
async fn index_page_no_post_when_no_posts() {
    BlogServer::empty()
        .start()
        .await
        .get("/")
        .await
        .expect()
        .status(200)
        .contains("No posts yet.")
        .verify()
        .await;
}

#[tokio::test]
async fn index_page_sorts_posts_by_date_newest_first() {
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
    let mut server = BlogServer::empty();
    for (path, content) in posts {
        server = server.add_file(path, content);
    }

    // Get the index page
    let response = server.start().await.get("/").await;

    // Check the response
    response
        .expect()
        .status(200)
        // First verify all posts are present
        .contains("Oldest Post")
        .contains("Middle Post (JS Date)")
        .contains("Newer Post (Short Format)")
        .contains("Newest Post (Full Format)")
        .contains("Undated Post Z")
        .contains("Another Undated Post A")
        // Then verify the order - we'll check adjacent pairs to confirm sorting
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
        ])
        .verify()
        .await;
}

mod specification_support {
    use blog_engine::create_app_with_content_dir;
    use std::fs;
    use std::net::SocketAddr;
    use tempfile::TempDir;

    pub struct BlogServer {
        content_on_server: Vec<FileOnServer>,
    }

    impl BlogServer {
        pub fn empty() -> Self {
            BlogServer {
                content_on_server: Vec::new(),
            }
        }

        pub fn with_file(target_path: &str, content: &str) -> Self {
            BlogServer {
                content_on_server: vec![FileOnServer {
                    target_path: target_path.to_string(),
                    content: content.to_string(),
                }],
            }
        }

        pub fn add_file(self, target_path: &str, content: &str) -> Self {
            let new_file = FileOnServer {
                target_path: target_path.to_string(),
                content: content.to_string(),
            };

            let mut content_on_server = self.content_on_server;
            content_on_server.push(new_file);

            BlogServer { content_on_server }
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

            let app = create_app_with_content_dir(&temp_path);
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
        app: axum::Router,
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
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_future)
                .await
                .unwrap();
        });

        (server_addr, shutdown_tx, server_handle)
    }

    pub struct ResponseExpectations {
        response: reqwest::Response,
        expectations: Vec<Box<dyn FnOnce(&str) + Send>>,
    }

    impl ResponseExpectations {
        pub fn new(response: reqwest::Response) -> Self {
            Self {
                response,
                expectations: Vec::new(),
            }
        }

        pub fn status(mut self, expected: u16) -> Self {
            let status = self.response.status().as_u16();
            self.expectations.push(Box::new(move |_| {
                assert_eq!(
                    status, expected,
                    "Expected status code {}, got {}",
                    expected, status
                );
            }));
            self
        }

        pub fn contains(mut self, substring: &str) -> Self {
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

        pub async fn verify(self) {
            let body = self
                .response
                .text()
                .await
                .expect("Failed to read response body");

            for expectation in self.expectations {
                expectation(&body);
            }
        }
    }
    pub trait IntoAssertion {
        fn expect(self) -> ResponseExpectations;
    }

    impl IntoAssertion for reqwest::Response {
        fn expect(self) -> ResponseExpectations {
            ResponseExpectations::new(self)
        }
    }

    struct FileOnServer {
        content: String,
        target_path: String,
    }
}
