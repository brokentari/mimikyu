
use axum::extract::ws::{WebSocket, Message};
use futures::{lock::Mutex, stream::{StreamExt, SplitSink, SplitStream}, SinkExt};
use tokio::{sync::broadcast};
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
    board: Mutex<Board>,
    tx: broadcast::Sender<Board>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessage {
    mode: Mode,
    x: u8,
    y: u8,
    color: String,
}

fn process_message(state: &Arc<AppState>, msg: EventMessage) {
    match msg.mode {
        Mode::Draw => {
            state.draw_pixel(msg.x, msg.y, msg.color)
        }, 
        Mode::Erase => {
            state.erase_pixel(msg.x, msg.y)
        }, 
        Mode::Clear => {
            state.clear();
        }
    }
}

fn spawn_send_task(mut sender: SplitSink<WebSocket, Message>, mut rx: broadcast::Receiver<Vec<Vec<u32>>>) -> tokio::task::JoinHandle<()>  {
    tokio::spawn(async move {
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
    })
}

fn spawn_recv_task(mut receiver: SplitStream<WebSocket>, state: Arc<AppState>)  -> tokio::task::JoinHandle<()> {
    
    tokio::spawn(async move {
        // we receive messages from the client
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(t) = message {
                match serde_json::from_str::<EventMessage>(&t) {
                    Ok(message) => {

                        process_message(&state, message);
                        
                        // broadcast the  new board to other subcribers
                        let _ = state.tx.send(state.board.try_lock().unwrap().clone());
                    }
                    Err(_) => {
                        println!(">>> Client sent another type of message {:?}", t);
                    }
                }
            }
        }
    })
}

pub async fn realtime_draw_stream(state: Arc<AppState>, ws: WebSocket) {
    let (sender, receiver) = ws.split();
    let rx = state.tx.subscribe();

    let v = state.board.try_lock().unwrap().clone();
    match state.tx.send(v) {
        Ok(_) => (), 
        Err(error) => panic!("No receivers available: {:?}", error)
    };

    let mut send_task = spawn_send_task(sender, rx);
    let mut recv_task = spawn_recv_task(receiver, state);

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

impl AppState {

    pub fn new() -> AppState {
        let (tx, _) = broadcast::channel::<Board>(10);
        AppState { board: Mutex::new(vec![vec![0 as u32; 32]; 16]) , tx }
    }

    fn draw_pixel(&self, x: u8, y:u8, color: String) {
        println!(">>> Drawing pixel at ({}, {}) with color {}", x, y, color);

        let color_string = color;
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

            self.board.try_lock().unwrap()[x as usize]
                [y as usize] = color_base;
    }

    fn erase_pixel(&self, x: u8, y: u8) {
        println!(">>> Erasing pixel at ({}, {})", x, y);
        self.board.try_lock().unwrap()[x as usize] [y as usize] = 0;
    }

    fn clear(&self) {
        println!(">>> Clearing canvas");
        self.board.try_lock().unwrap().iter_mut().for_each(
            |row| row.iter_mut().for_each(|pixel| *pixel = 0),
        );
    }
} 