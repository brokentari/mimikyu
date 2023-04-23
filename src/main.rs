use std::sync::Arc;
use axum::{
    routing::get,
    Router, Server,
};
use tower_http::services::ServeFile;
use crate::{handlers::Handlers, state::AppState};

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

    let graceful_server = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("unable to install CTRL+C signal handler");
    });

    println!("Listening on http://{}", local_addr);

    if let Err(e) = graceful_server.await {
        eprintln!("server error: {}", e);
    }
    
    println!("\nexiting mimikyu server...");
    let matrix_guard = state::get_matrix().lock().unwrap();
    
    matrix_guard.matrix.canvas().clear();

    std::mem::drop(matrix_guard);
}

