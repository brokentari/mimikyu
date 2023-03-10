### Prerequisites
- Rust 1.68+

# Build from Source

**Make sure to have Rust 1.68 or greater installed. For instructions on installing Rust, visit https://www.rust-lang.org/.**

1. Clone the repo.

`git clone git@github.com:brokentari/mimikyu.git"`

2. Build the project with appropriate target.

`cargo build (optional: --release)`

3. Run the built target.

```cargo run```

4. From a seperate client, open a browser and visit ```localhost:7032```.


# Usage
This application is supposed to be ran by a computer with a RGB LED matrix panel (see [Adafruit RGB LED Panel](https://www.adafruit.com/product/420)) attached to it. 

The computer will listen in for any events (draw, erase, clear, etc.) from multiple clients, which the computer running this server will appropriately relay to the LED panel to display the same pixels as in the web client.

