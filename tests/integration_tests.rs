use specification_support::IntoAssertion;

use crate::specification_support::BlogServer;

#[tokio::test]
async fn test_happy_endpoint_returns_200() {
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
async fn test_can_serve_single_blog_post() {
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
async fn test_can_serve_single_blog_post_with_slug_from_frontmatter() {
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
async fn test_can_serve_blog_post_without_front_matter() {
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
