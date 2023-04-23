
use axum::extract::ws::{WebSocket, Message};
use futures::{lock::Mutex, stream::{StreamExt, SplitSink, SplitStream}, SinkExt};
use tokio::sync::broadcast;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rpi_led_matrix::{LedMatrix, LedColor, LedMatrixOptions, LedCanvas, LedRuntimeOptions};

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


pub struct Matrix {
    pub matrix: LedMatrix, 
}

static mut MATRIX: Option<std::sync::Mutex<Matrix>> = None;

static ONCE: std::sync::Once = std::sync::Once::new(); 

pub fn get_matrix() -> &'static std::sync::Mutex<Matrix> {
    ONCE.call_once(|| {
        let mut matrix_options = LedMatrixOptions::new();
        matrix_options.set_rows(16);
        matrix_options.set_cols(32);
        matrix_options.set_hardware_mapping("adafruit-hat");
        matrix_options.set_limit_refresh(60);
        matrix_options.set_refresh_rate(false);

        let mut matrix_runtime_options = LedRuntimeOptions::new();
        matrix_runtime_options.set_gpio_slowdown(4);
        
        let matrix = LedMatrix::new(Some(matrix_options), Some(matrix_runtime_options)).unwrap();
        let singleton = std::sync::Mutex::new(Matrix{matrix});

        unsafe { MATRIX = Some(singleton) };
    });

    unsafe { MATRIX.as_ref().unwrap() }
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
    let tx = state.tx.clone();
    tokio::spawn(async move {
        // we receive messages from the client
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(t) = message {
                match serde_json::from_str::<EventMessage>(&t) {
                    Ok(message) => {

                        process_message(&state, message);
                        
                        // broadcast the  new board to other subcribers
                        let _ = tx.send(state.board.try_lock().unwrap().clone());
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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {

    pub fn new() -> AppState {
        let (tx, _) = broadcast::channel::<Board>(10);
        get_matrix();

        AppState{ board: Mutex::new(vec![vec![0_u32; 32]; 16]) , tx }
    }

    fn draw_pixel(&self, x: u8, y:u8, color: String) {
        println!(">>> Drawing pixel at ({}, {}) with color {}", x, y, color);

        let color_string = color;
        let colors = color_string[4..color_string.len() - 1]
            .split(',')
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|x| x.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        // let mut color_base: u32 = 0b0000_0000_0000_0000_0000_0000;

        // for i in 0..colors.len() {
        //    color_base += colors[i] << (8 * (colors.len() - i - 1));
        // }

        let led_matrix = get_matrix().lock().unwrap();
        let red = *colors.get(0).unwrap();
        let green = *colors.get(1).unwrap();
        let blue = *colors.get(2).unwrap();

        println!("red: {}, green: {}, blue: {}", red, green, blue);
             
        let mut canvas = led_matrix.matrix.canvas();
        canvas.set(y.into(), x.into(), &LedColor { red, green, blue });

        //self.board.try_lock().unwrap()[x as usize]
        //   [y as usize] = color_base;
    }

    fn erase_pixel(&self, x: u8, y: u8) {
        println!(">>> Erasing pixel at ({}, {})", x, y);
        let led_matrix = get_matrix().lock().unwrap();
        let mut canvas = led_matrix.matrix.canvas();
        canvas.set(y.into(), x.into(), &LedColor { red: 0, green: 0, blue: 0 });
        self.board.try_lock().unwrap()[x as usize] [y as usize] = 0;
    }

    fn clear(&self) {
        println!(">>> Clearing canvas");

        let led_matrix = get_matrix().lock().unwrap();
        let mut canvas = led_matrix.matrix.canvas();
        canvas.clear();
        self.board.try_lock().unwrap().iter_mut().for_each(
            |row| row.iter_mut().for_each(|pixel| *pixel = 0),
        );
    }
} 
