mod app;
mod network;
mod renderer;
mod ui;

fn main() {
    pollster::block_on(app::run());
}
