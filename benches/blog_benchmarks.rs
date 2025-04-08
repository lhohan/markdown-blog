use blog_engine::create_app_with_content_dir;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::net::SocketAddr;
use tempfile::TempDir;
use tokio::sync::oneshot;

struct TestServer {
    addr: SocketAddr,
    _temp_dir: TempDir, // Keep dir alive
    _shutdown: oneshot::Sender<()>,
}

impl TestServer {
    async fn new(content_dir: TempDir) -> Self {
        let app = create_app_with_content_dir(content_dir.path());
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let shutdown_future = async {
            let _ = shutdown_rx.await;
        };

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_future)
                .await
                .unwrap();
        });

        TestServer {
            addr: server_addr,
            _temp_dir: content_dir,
            _shutdown: shutdown_tx,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}

fn setup_test_environment(num_posts: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let posts_dir = temp_dir.path().join("posts");
    fs::create_dir_all(&posts_dir).unwrap();

    for i in 0..num_posts {
        let content = format!(
            r#"---
title: Test Post {i}
datePublished: 2024-01-{:02}
---
# Post {i}

This is test content for post {i}."#,
            (i % 28) + 1
        );

        let file_path = posts_dir.join(format!("test-post-{i}.md"));
        fs::write(file_path, content).unwrap();
    }

    temp_dir
}

fn benchmark_blog(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let server = runtime.block_on(async {
        let temp_dir = setup_test_environment(10_000);
        TestServer::new(temp_dir).await
    });

    let mut group = c.benchmark_group("blog_operations");

    // Benchmark homepage (all posts)
    group.bench_function("get_all_posts_10k", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let client = reqwest::Client::new();
                client.get(server.url("/")).send().await.unwrap();
            });
        })
    });

    // Benchmark single post access
    group.bench_function("get_single_post_from_10k", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let client = reqwest::Client::new();
                client
                    .get(server.url("/test-post-5000"))
                    .send()
                    .await
                    .unwrap();
            });
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_blog);
criterion_main!(benches);
