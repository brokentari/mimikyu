
use axum::extract::ws::{WebSocket, Message};
use futures::{lock::Mutex, stream::StreamExt, SinkExt};
use tokio::sync::broadcast;
use std::{sync::Arc};
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Mode {
    Draw,
    Erase,
    Clear,
}

pub type Board = Vec<Vec<u32>>;

pub struct AppState {
    pub board: Mutex<Board>,
    pub tx: broadcast::Sender<Board>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessage {
    mode: Mode,
    x: u8,
    y: u8,
    color: String,
}


impl AppState {

  pub async fn realtime_draw_stream(&self, app_state: Arc<AppState>, ws: WebSocket) {
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
}