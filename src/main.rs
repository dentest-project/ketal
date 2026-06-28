use axum::serve;
use jsonrpc_usecase::axum as jsonrpc_axum;
use ketal::build_service;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let service = build_service().expect("service registration should succeed");
    let app = jsonrpc_axum::router(service);
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind TCP listener");

    println!("JSON-RPC server listening on http://127.0.0.1:3000/rpc");
    serve(listener, app).await.expect("HTTP server failed");
}
