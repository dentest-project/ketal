use axum::{
    http::{
        Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    serve,
};
use jsonrpc_usecase::axum as jsonrpc_axum;
use ketal::{
    build_service,
    infrastructure::config::{cors_allowed_origin, server_bind_address},
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let service = build_service().expect("service registration should succeed");
    let cors = CorsLayer::new()
        .allow_origin(cors_allowed_origin())
        .allow_methods([Method::POST])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    let app = jsonrpc_axum::router(service).layer(cors);
    let bind_addr = server_bind_address();
    let listener = TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|error| panic!("failed to bind TCP listener on {bind_addr}: {error}"));
    let listen_addr = listener
        .local_addr()
        .expect("TCP listener should expose its local address");

    println!("JSON-RPC server listening on http://{listen_addr}/rpc");
    serve(listener, app).await.expect("HTTP server failed");
}
