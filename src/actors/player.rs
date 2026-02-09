use macroquad::prelude::*;

// 玩家实体：位置与移动速度
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub pos: Vec2,
    pub speed: f32,
}

impl Player {
    // 创建玩家
    pub fn new(spawn: Vec2) -> Self {
        Self {
            pos: spawn,
            speed: 180.0,
        }
    }
}
