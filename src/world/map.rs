use serde::Deserialize;

// 地图配置集合（数据驱动入口）
#[derive(Clone, Debug, Deserialize)]
pub struct MapsConfig {
    pub start_map: String,
    pub maps: Vec<MapConfig>,
}

impl Default for MapsConfig {
    // 默认地图数据：用于缺失配置文件时的兜底
    fn default() -> Self {
        Self {
            start_map: "qingtian".to_string(),
            maps: vec![
                MapConfig {
                    id: "qingtian".to_string(),
                    name: "青天坊市".to_string(),
                    spawn: Vec2Def::new(120.0, 280.0),
                    portals: vec![PortalConfig {
                        pos: Vec2Def::new(820.0, 280.0),
                        radius: 26.0,
                        to_map: "yanling".to_string(),
                        to_pos: Vec2Def::new(120.0, 260.0),
                        is_unlocked: true,
                    }],
                    bosses: vec!["坊市守卫".to_string()],
                },
                MapConfig {
                    id: "yanling".to_string(),
                    name: "燕翎台".to_string(),
                    spawn: Vec2Def::new(120.0, 260.0),
                    portals: vec![PortalConfig {
                        pos: Vec2Def::new(820.0, 260.0),
                        radius: 26.0,
                        to_map: "tianyi".to_string(),
                        to_pos: Vec2Def::new(120.0, 300.0),
                        is_unlocked: true,
                    }],
                    bosses: vec!["赤焰兽".to_string()],
                },
                MapConfig {
                    id: "tianyi".to_string(),
                    name: "天一城".to_string(),
                    spawn: Vec2Def::new(120.0, 300.0),
                    portals: vec![PortalConfig {
                        pos: Vec2Def::new(820.0, 300.0),
                        radius: 26.0,
                        to_map: "qingtian".to_string(),
                        to_pos: Vec2Def::new(120.0, 280.0),
                        is_unlocked: true,
                    }],
                    bosses: vec!["青鳞王".to_string()],
                },
            ],
        }
    }
}

// 序列化用的二维坐标
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}

impl Vec2Def {
    // 创建坐标
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    // 转换为 macroquad::Vec2
    pub fn to_vec2(self) -> macroquad::prelude::Vec2 {
        macroquad::prelude::vec2(self.x, self.y)
    }
}

// 单张地图配置
#[derive(Clone, Debug, Deserialize)]
pub struct MapConfig {
    pub id: String,
    pub name: String,
    pub spawn: Vec2Def,
    pub portals: Vec<PortalConfig>,
    pub bosses: Vec<String>,
}

// 传送点配置
#[derive(Clone, Debug, Deserialize)]
pub struct PortalConfig {
    pub pos: Vec2Def,
    pub radius: f32,
    pub to_map: String,
    pub to_pos: Vec2Def,
    pub is_unlocked: bool,
}

impl PortalConfig {
    // 判断玩家是否进入传送点范围
    pub fn contains(&self, pos: macroquad::prelude::Vec2) -> bool {
        pos.distance(self.pos.to_vec2()) <= self.radius
    }
}
