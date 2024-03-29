### Prerequisites
- Rust 1.68+
- Raspberry Pi
- RBG LED Matrix (e.g [adafruit 16x32 panel](https://www.adafruit.com/product/420))
- [adafruit Matrix HAT](https://www.adafruit.com/product/2345)

# Mimikyu

A Websocket and Rust implementation of a real-time 64x32 whiteboard that is used to represent a RGB matrix board powered by a Raspberry Pi. 

![website](/images/mimikyu_website.png)
![demo](/images/mimikyu_demo.gif)

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
This program can only be compiled for and ran by a Raspberry PI due to the requirements of the `rpi-led-matrix` crate, which contains Rust bindings for a C++ library [`rpi-rgb-led-matrix`](https://github.com/hzeller/rpi-rgb-led-matrix) "to control RGB LED panels with the Raspberry Pi."

The computer will listen in for any events (draw, erase, clear, etc.) from multiple clients, which the computer running this server will appropriately relay to the LED panel to display the same pixels as in the web client.

