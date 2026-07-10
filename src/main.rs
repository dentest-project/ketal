use axum::serve;
use jsonrpc_usecase::axum as jsonrpc_axum;
use ketal::{build_service, infrastructure::config::server_bind_address};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let service = build_service().expect("service registration should succeed");
    let app = jsonrpc_axum::router(service);
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
