use blog_engine::create_app;
use std::net::SocketAddr;

use blog_engine::create_app_with_content_dir;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_happy_endpoint_returns_200() {
    // Start the server
    let app = create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    // Spawn the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a request to the /happy endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/health", server_addr))
        .send()
        .await
        .expect("Failed to send request");

    // Assert that the response has a 200 status code
    assert_eq!(response.status().as_u16(), 200);

    // Assert that the response body is "I'm happy!"
    let body = response.text().await.expect("Failed to get response body");
    assert_eq!(body, "I'm ok!");
}

#[tokio::test]
async fn test_can_serve_single_blog_post() {
    // Create a temporary directory for test blog posts
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test blog post with front matter
    let post_content = r#"---
title: Test Post
date: 2023-01-01
---
# Hello World

This is a test blog post.
"#;
    fs::create_dir_all(temp_path.join("posts")).unwrap();
    fs::write(temp_path.join("posts/test-post.md"), post_content).unwrap();

    // Start the server with our content directory
    let app = create_app_with_content_dir(temp_path);
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a request to the post endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/test-post", server_addr))
        .send()
        .await
        .expect("Failed to send request");

    // Assert that the response has a 200 status code
    assert_eq!(response.status().as_u16(), 200);

    // Assert that the response body contains our post content
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Hello World"));
    assert!(body.contains("This is a test blog post."));
}

#[tokio::test]
async fn test_can_serve_single_blog_post_with_slug_from_frontmatter() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let post_content = r#"---
title: Test Post
date: 2023-01-01
slug: abc123
---
# Hello World

This is a test blog post.
"#;
    fs::create_dir_all(temp_path.join("posts")).unwrap();
    fs::write(temp_path.join("posts/test-post.md"), post_content).unwrap();

    // Start the server with our content directory
    let app = create_app_with_content_dir(temp_path);
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a request to the post endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/abc123", server_addr))
        .send()
        .await
        .expect("Failed to send request");

    // Assert that the response has a 200 status code
    assert_eq!(response.status().as_u16(), 200);

    // Assert that the response body contains our post content
    let body = response.text().await.expect("Failed to get response body");
    assert!(body.contains("Hello World"));
    assert!(body.contains("This is a test blog post."));
}
