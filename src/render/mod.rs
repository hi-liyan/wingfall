use macroquad::prelude::*;

use crate::actors::player::Player;
use crate::ui::{draw_centered_text, draw_text_ui, Ui};
use crate::world::map::MapConfig;

// 绘制地图背景与名称
pub fn draw_map(ui: &Ui, map: &MapConfig) {
    clear_background(Color::new(0.05, 0.05, 0.08, 1.0));
    draw_centered_text(ui, &map.name, 80.0, 36, WHITE);
}

// 绘制传送点位置
pub fn draw_portals(_ui: &Ui, map: &MapConfig) {
    for portal in &map.portals {
        let pos = portal.pos.to_vec2();
        draw_circle_lines(pos.x, pos.y, portal.radius, 2.0, SKYBLUE);
        draw_circle(pos.x, pos.y, 4.0, SKYBLUE);
    }
}

// 绘制玩家
pub fn draw_player(_ui: &Ui, player: &Player) {
    draw_circle(player.pos.x, player.pos.y, 6.0, YELLOW);
}

// 绘制HUD信息
pub fn draw_hud(ui: &Ui, _map: &MapConfig) {
    let hint = "E: 传送  方向键/WASD 移动";
    draw_text_ui(ui, hint, 16.0, 520.0, 20, GRAY);
}
