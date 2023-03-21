use std::sync::Arc;

use axum::{
  response::{IntoResponse, Html}, 
  http::Response, 
  extract::{WebSocketUpgrade, State}};

use crate::state::{AppState, realtime_draw_stream};

pub struct Handlers {
  
}

impl Handlers {
  pub async fn get_root() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/frontend/index.html").await.unwrap();

    Html(markup)
  }

  pub async fn get_javascript() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/frontend/index.mjs").await.unwrap();

        Response::builder()
            .header("content-type", "application/javascript; charset=utf-8")
            .body(markup)
            .unwrap()
    }

  pub async fn get_css() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/frontend/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css; charset=utf-8")
        .body(markup)
        .unwrap()
  }

  pub async fn get_realtime_stream(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |ws| realtime_draw_stream(state, ws))
  }
}