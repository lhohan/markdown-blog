use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = blog_engine::create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
