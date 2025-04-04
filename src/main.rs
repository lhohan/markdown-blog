use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_utc_timestamps()
        .with_colors(true)
        .init()
        .unwrap();

    let app = blog_engine::create_app_with_defaults();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    log::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
