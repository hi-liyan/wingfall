mod app;
mod assets;
mod config;
mod render;
mod systems;
mod ui;
mod world;
mod actors;
mod items;

use crate::config::window_conf;

#[macroquad::main(window_conf)]
// 程序入口：初始化窗口配置并启动游戏主循环
async fn main() {
    app::run().await;
}
