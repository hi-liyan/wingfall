use macroquad::prelude::*;

use crate::actors::player::Player;
use crate::world::World;

// 处理玩家移动输入
pub fn handle_movement(player: &mut Player) {
    let mut dir = vec2(0.0, 0.0);
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        dir.x += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        dir.y += 1.0;
    }
    if dir.length_squared() > 0.0 {
        dir = dir.normalize();
    }
    player.pos += dir * player.speed * get_frame_time();

    // 简单边界限制
    player.pos.x = player.pos.x.clamp(12.0, crate::config::SCREEN_W - 12.0);
    player.pos.y = player.pos.y.clamp(12.0, crate::config::SCREEN_H - 12.0);
}

// 处理交互输入（当前仅传送点）
pub fn handle_interaction(world: &mut World, player: &mut Player) {
    if !is_key_pressed(KeyCode::E) {
        return;
    }

    // 若在传送点范围内则切换地图
    if let Some((target_map, target_pos)) = world.try_teleport(player.pos) {
        player.pos = target_pos;
        world.switch_map(target_map);
    }
}
