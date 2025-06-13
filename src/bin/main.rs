use axum::Router;
use std::env;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_utc_timestamps()
        .with_colors(true)
        .init()
        .unwrap_or_else(|e| println!("Error configuring logger: {e}"));

    let content_dir = env::var("BLOG_CONTENT_DIR").unwrap_or_else(|_| "content".to_string());
    let blog_dir = env::var("BLOG_DIR").unwrap_or_else(|_| "content".to_string());

    let app: Router = blog_engine::create_app_with_dirs(&content_dir, &blog_dir);

    let host: IpAddr = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .expect("Invalid HOST address");

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("Invalid PORT number");

    let addr = SocketAddr::new(host, port);
    let listener = TcpListener::bind(addr).await?;

    println!("Blog engine running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
