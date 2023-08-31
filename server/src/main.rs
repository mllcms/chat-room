use axum::routing::get;
use axum::Router;
use chat_demo::{websocket, UsersState};
use mll_axum_utils::middleware::logger::Logger;
use mll_axum_utils::utils::echo_ip_addrs;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:80".parse().unwrap();
    echo_ip_addrs(&addr);
    let view_server = ServeDir::new("dist").append_index_html_on_directories(true);

    let app = Router::new()
        .nest_service("/", view_server)
        .route("/ws", get(websocket))
        .with_state(UsersState::default())
        .layer(Logger::default());

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}