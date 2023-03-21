use std::{sync::Arc};

use axum::{
    routing::get,
    Router, Server,
};
use tower_http::services::ServeFile;
use crate::{handlers::Handlers, state::{AppState}};

pub mod handlers;
pub mod state;

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState::new());

    let router = Router::new()
        .nest_service("/images/favicon.ico", ServeFile::new("src/frontend/favicon.ico"))
        .route("/", get(Handlers::get_root))
        .route("/index.mjs", get(Handlers::get_javascript))
        .route("/index.css", get(Handlers::get_css))
        .route("/realtime/draw", get(Handlers::get_realtime_stream))
        .with_state(app_state);


    let server = Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(router.into_make_service());
    let local_addr = server.local_addr();

    println!("Listening on http://{}", local_addr);

    server.await.unwrap();
}

