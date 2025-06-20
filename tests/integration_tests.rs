use rstest::rstest;

use crate::specification_support::BlogServer;

#[tokio::test]
async fn health_endpoint_should_return_200() {
    BlogServer::new()
        .get("/health")
        .expect_status_code(200)
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_be_accessible_via_filename() {
    let post_content = ""; // content not relevant for test

    BlogServer::with_file("posts/test-post.md", post_content)
        .get("/test-post")
        .expect_status_code(200)
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_be_accessible_via_slug_when_slug_in_frontmatter() {
    let post_content = "---
slug: hello
---
";

    BlogServer::with_file("posts/test-post.md", post_content)
        .get("/hello")
        .expect_status_code(200)
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_render_markdown_content() {
    let post_content = "---
slug: hello
---
This is a test blog post.
";

    BlogServer::with_file("posts/test-post.md", post_content)
        .get("/hello")
        .expect_body_contains("This is a test blog post.")
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_display_title_when_title_in_frontmatter() {
    let post_content = "---
title: Hello World
slug: hello
---
";

    BlogServer::with_file("posts/test-post.md", post_content)
        .get("/hello")
        .expect_body_contains("Hello World")
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_display_date_when_date_in_frontmatter() {
    let post_content = "---
datePublished: 2023-01-01
slug: hello
---
";

    BlogServer::with_file("posts/test-post.md", post_content)
        .get("/hello")
        .expect_status_code(200)
        .expect_body_contains("Published on January 01, 2023")
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_return_404_when_frontmatter_malformed() {
    let post_content = "---
slug: hello
"; // missing closing delimiter

    BlogServer::with_file("posts/test-post.md", post_content)
        .get("/hello")
        .expect_status_code(404)
        .execute()
        .await;
}

#[tokio::test]
async fn post_should_be_accessible_via_filename_when_no_frontmatter() {
    let post_content = "";

    BlogServer::with_file("posts/no-front-matter.md", post_content)
        .get("/no-front-matter")
        .expect_status_code(200)
        .execute()
        .await;
}

#[tokio::test]
async fn server_should_return_404_when_post_not_found() {
    BlogServer::new()
        .get("/non-existent-post")
        .expect_status_code(404)
        .execute()
        .await;
}

#[tokio::test]
async fn index_should_display_posts() {
    let post1 = "---
title: Post One
---
";

    let post2 = "---
title: Post Two
---
";

    BlogServer::new()
        .add_file("posts/first-post.md", post1)
        .add_file("posts/second-post.md", post2)
        .get("/")
        .expect_body_contains("Post One")
        .expect_body_contains("Post Two")
        .execute()
        .await;
}

#[tokio::test]
async fn index_should_display_empty_message_when_no_posts() {
    BlogServer::new()
        .get("/")
        .expect_body_contains("No posts yet")
        .execute()
        .await;
}

#[tokio::test]
async fn index_should_sort_posts_by_date_descending() {
    let posts = [
        (
            "posts/oldest.md",
            "---\ntitle: Oldest Post\ndatePublished: 2020-01-01\n---\n",
        ),
        (
            "posts/middle.md",
            "---\ntitle: Middle Post\ndatePublished: 2021-01-01\n---\n",
        ),
        (
            "posts/newest.md",
            "---\ntitle: Newest Post\ndatePublished: 2022-01-01\n---\n",
        ),
        ("posts/undated-z.md", "---\ntitle: Undated Z\n---\n"),
        ("posts/undated-a.md", "---\ntitle: Undated A\n---\n"),
    ];

    let mut server = BlogServer::new();
    for (path, content) in posts {
        server = server.add_file(path, content);
    }

    server
        .get("/")
        .expect_body_contains("Newest Post")
        .expect_body_contains("Oldest Post")
        .expect_contains_in_order(&[
            "Newest Post",
            "Middle Post",
            "Oldest Post",
            "Undated A",
            "Undated Z",
        ])
        .execute()
        .await;
}

#[tokio::test]
#[rstest]
async fn all_should_display_custom_title_when_configured(
    #[values(
        CustomTitleTestSetup::index(),
        CustomTitleTestSetup::post(),
        CustomTitleTestSetup::page()
    )]
    setup: CustomTitleTestSetup,
) {
    let config = "
site_title: \"My Custom Blog Title\"
site_description: \"A custom blog description\"
";

    setup
        .server
        .with_config(config)
        .get(&setup.slug)
        .expect_body_contains("My Custom Blog Title")
        .expect_not_contains("Your Blog")
        .execute()
        .await;
}

struct CustomTitleTestSetup {
    server: BlogServer,
    slug: String,
}

impl CustomTitleTestSetup {
    fn index() -> Self {
        Self {
            server: BlogServer::new(),
            slug: "/".to_string(),
        }
    }
    fn post() -> Self {
        Self {
            server: BlogServer::with_file("posts/post.md", ""),
            slug: "/post".to_string(),
        }
    }
    fn page() -> Self {
        Self {
            server: BlogServer::with_file("pages/page.md", ""),
            slug: "/p/page".to_string(),
        }
    }
}

#[tokio::test]
async fn page_should_be_accessible_via_p_prefix() {
    let page_content = "# Test Page

Content here.";

    BlogServer::new()
        .add_file("pages/about.md", page_content)
        .get("/p/about")
        .expect_body_contains("<h1>Test Page</h1>")
        .execute()
        .await;
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

    // ===== 1. PUBLIC (TEST) API TYPES =====

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

        pub fn get(self, path: &str) -> Obtained {
            Obtained {
                server: self, // BlogServer now directly owns itself in Obtained
                path: path.to_string(),
                assertions: Vec::new(),
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

        async fn start(self) -> RunningServer {
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

            RunningServer::new(server_addr, shutdown_tx, server_handle, temp_dir)
        }
    }

    pub struct Obtained {
        server: BlogServer,
        path: String,
        assertions: Vec<Box<dyn FnOnce(&Response) + Send>>,
    }

    impl Obtained {
        pub fn expect_status_code(mut self, expected: u16) -> Self {
            self.assertions.push(Box::new(move |response| {
                assert_eq!(
                    response.status_code, expected,
                    "Expected status code {}, got {}",
                    expected, response.status_code
                );
            }));
            self
        }

        pub fn expect_body_contains(mut self, text: &str) -> Self {
            let text = text.to_string(); // Clone to move into the closure
            self.assertions.push(Box::new(move |response| {
                assert!(
                    response.body.contains(&text),
                    "Response body does not contain '{}'. Full body:\n{}",
                    text,
                    response.body
                );
            }));
            self
        }

        pub fn expect_not_contains(mut self, text: &str) -> Self {
            let text = text.to_string(); // Clone to move into the closure
            self.assertions.push(Box::new(move |response| {
                assert!(
                    !response.body.contains(&text),
                    "Response body should not contain '{}'. Full body:\n{}",
                    text,
                    response.body
                );
            }));
            self
        }

        pub fn expect_contains_in_order(mut self, items: &[&str]) -> Self {
            // TODO: Review this implementation. Seems to work but copy-pasted from Claude.
            let items: Vec<String> = items.iter().map(|s| s.to_string()).collect();
            self.assertions.push(Box::new(move |response| {
                    let mut last_pos = 0;

                    for (i, substring) in items.iter().enumerate() {
                        match response.body[last_pos..].find(substring) {
                            Some(pos) => {
                                last_pos += pos + substring.len();
                            }
                            None => {
                                panic!(
                                    "Expected to find '{}' after position {}. Items should appear in order: {:?}. Full body:\n{}",
                                    substring, last_pos, items, response.body
                                );
                            }
                        }

                        if i < items.len() - 1 {
                            let next = &items[i + 1];
                            if response.body[last_pos..].find(next).is_none() {
                                panic!(
                                    "Expected to find '{}' after '{}', but it was not found. Full body:\n{}",
                                    next, substring, response.body
                                );
                            }
                        }
                    }
                }));
            self
        }

        pub async fn execute(self) {
            let server = self.server.start().await;
            let http_response = server.get(&self.path).await;

            let response = Response {
                status_code: http_response.status().as_u16(),
                body: http_response
                    .text()
                    .await
                    .expect("Failed to read response body"),
            };

            // Run all assertions collected
            for assertion in self.assertions {
                assertion(&response);
            }
        }
    }

    // ===== 2. INTERNAL SERVER TYPES =====

    struct RunningServer {
        server_addr: SocketAddr,
        shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
        server_handle: Option<tokio::task::JoinHandle<()>>,
        _temp_dir: TempDir,
    }

    impl RunningServer {
        fn new(
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

        async fn get(&self, path: &str) -> reqwest::Response {
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

    // ===== 3. HELPER TYPES =====

    struct Response {
        body: String,
        status_code: u16,
    }

    struct FileOnServer {
        content: String,
        target_path: String,
    }

    // ===== 4. HELPER FUNCTIONS =====

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
}
