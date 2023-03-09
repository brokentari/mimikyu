use std::{sync::Arc, str::FromStr};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Router, Server,
};
use futures::{lock::Mutex, stream::StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::services::ServeFile;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum Mode {
    Draw,
    Erase,
    Clear,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventMessage {
    mode: Mode,
    x: u8,
    y: u8,
    color: String,
}

type Board = Vec<Vec<u32>>;

struct AppState {
    board: Mutex<Board>,
    tx: broadcast::Sender<Board>,
}

#[tokio::main]
async fn main() {
    let board = Mutex::new(vec![vec![0 as u32; 32]; 16]);
    let (tx, _rx) = broadcast::channel::<Board>(10);

    //let app_state = AppState { tx: tx.clone()};
    let app_state = Arc::new(AppState { board, tx });

    let router = Router::new()
        .nest_service("/images/favicon.ico", ServeFile::new("src/favicon.ico"))
        .route("/", get(root_get))
        .route("/index.mjs", get(get_js))
        .route("/index.css", get(get_css))
        .route("/realtime/draw", get(get_realtime_draw))
        .with_state(app_state);

    let server = Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(router.into_make_service());
    let local_addr = server.local_addr();

    println!("Listening on http://{}", local_addr);

    server.await.unwrap();
}

async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();

    Html(markup)
}

async fn get_js() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(markup)
        .unwrap()
}

async fn get_css() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css; charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn get_realtime_draw(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws| realtime_draw_stream(state, ws))
}

async fn realtime_draw_stream(app_state: Arc<AppState>, ws: WebSocket) {
    let (mut sender, mut receiver) = ws.split();

    let mut rx = app_state.tx.subscribe();

    let v = app_state.board.try_lock().unwrap().clone();
    let _ = app_state.tx.send(v);

    let mut send_task = tokio::spawn(async move {
        // we receive board information through broadcast channel
        while let Ok(msg) = rx.recv().await {
            // we send the board information to the client
            if sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let tx = app_state.tx.clone();

    let mut recv_task =
        tokio::spawn(async move {
            // we receive messages from the client
            while let Some(Ok(message)) = receiver.next().await {
                if let Message::Text(t) = message {
                    match serde_json::from_str::<EventMessage>(&t) {
                        Ok(message) => {
                            match message.mode {
                                Mode::Draw => {
                                    println!(">>> Drawing pixel at ({}, {}) with color {}", message.x, message.y, message.color);
                                    let color_string = message.color;
                                    let colors = color_string[4..color_string.len() - 1]
                                        .split(',')
                                        .collect::<Vec<&str>>()
                                        .into_iter()
                                        .map(|x| x.parse::<u32>().unwrap())
                                        .collect::<Vec<u32>>();

                                    let mut color_base: u32 = 0b0000_0000_0000_0000_0000_0000;

                                    for i in 0..colors.len() {
                                        color_base += colors[i] << (8 * (colors.len() - i - 1));
                                    }

                                    app_state.board.try_lock().unwrap()[message.x as usize]
                                        [message.y as usize] = color_base;
                                }
                                Mode::Erase => {
                                    println!(">>> Erasing pixel at ({}, {})", message.x, message.y);
                                    app_state.board.try_lock().unwrap()[message.x as usize]
                                        [message.y as usize] = 0;
                                }
                                Mode::Clear => {
                                    println!(">>> Clearing canvas");
                                    app_state.board.try_lock().unwrap().iter_mut().for_each(
                                        |row| row.iter_mut().for_each(|pixel| *pixel = 0),
                                    );
                                }
                            }
                            // broadcast the  new board to other subcribers
                            let _ = tx.send(app_state.board.try_lock().unwrap().clone());
                        }
                        Err(_) => {
                            println!(">>> Client sent another type of message {:?}", t);
                        }
                    }
                }
            }
        });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

// fn process_websocket_message(app_state: AppState, msg: Message) -> ControlFlow<(), ()> {
//     match msg {
//         Message::Text(t) => {
//             match serde_json::from_str::<EventMessage>(&t) {
//                 Ok(message) => {
//                     match message.mode {
//                         Mode::Draw => {
//                             println!(">>> Drawing pixel at ({}, {})", message.x, message.y);
//                             // app_state.tx.send(value)
//                             // app_state.board[message.x as usize][message.y as usize] = 1;
//                         }
//                         Mode::Erase => {
//                             println!(">>> Erasing pixel at ({}, {})", message.x, message.y);
//                             // app_state.board[message.x as usize][message.y as usize] = 0;
//                         }
//                         Mode::Clear => {
//                             println!(">>> Clearing canvas");
//                             app_state.tx.send(vec![vec![0; 32]; 16]).unwrap();
//                             // app_state.board = vec![vec![0; 32]; 16];
//                         }
//                     }
//                 }
//                 Err(_) => {
//                     println!(">>> Client sent another type of message {:?}", t);
//                 }
//             }
//          }
//         Message::Binary(d) => { println!(">>> Client sent {} bytes: {:?}", d.len(), d); }
//         Message::Close(c) => {
//             if let Some(cf) = c {
//                 println!(">>> Client sent code {} and reason {}", cf.code, cf.reason);
//             } else {
//                 println!(">>> Client somehow sent close message without CloseFrame");
//             }

//             return ControlFlow::Break(());
//         }
//         Message::Pong(v) => { println!(">>> Client sent pong with {:?}", v); }
//         Message::Ping(v) => { println!(">>> Client sent ping with {:?}", v); }
//     }

//     ControlFlow::Continue(())
// }
