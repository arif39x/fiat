mod animation;
mod app;
mod core;
mod network;
mod render;
mod ui;

fn main() {
    pollster::block_on(app::run());
}
