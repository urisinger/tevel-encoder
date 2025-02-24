use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Define the static files directory
    let static_dir = ServeDir::new("static");

    // Create the router
    let app = Router::new()
        .nest_service("/", static_dir) // Serve files under /assets/
        ;

    // Define the address
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
