use std::collections::HashMap;
use std::fs;

use macroquad::prelude::*;

use crate::world::map::{MapConfig, MapsConfig};

pub mod map;

// 世界状态：当前地图与地图表
pub struct World {
    current: String,
    maps: HashMap<String, MapConfig>,
}

impl Default for World {
    // 默认世界：内置三张地图与两处传送点（闭环示例）
    fn default() -> Self {
        let config = MapsConfig::default();
        let maps = config
            .maps
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();
        Self {
            current: config.start_map,
            maps,
        }
    }
}

impl World {
    // 从JSON文件加载地图配置
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let config: MapsConfig = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let maps = config
            .maps
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();
        Ok(Self {
            current: config.start_map,
            maps,
        })
    }

    // 获取当前地图配置
    pub fn current_map(&self) -> &MapConfig {
        self.maps
            .get(&self.current)
            .expect("current map missing")
    }

    // 获取当前地图出生点
    pub fn current_spawn(&self) -> Vec2 {
        self.current_map().spawn.to_vec2()
    }

    // 切换当前地图
    pub fn switch_map(&mut self, map_id: String) {
        if self.maps.contains_key(&map_id) {
            self.current = map_id;
        }
    }

    // 传送点判定：在范围内并且解锁则返回目标地图与目标坐标
    pub fn try_teleport(&self, pos: Vec2) -> Option<(String, Vec2)> {
        for portal in &self.current_map().portals {
            if portal.is_unlocked && portal.contains(pos) {
                return Some((portal.to_map.clone(), portal.to_pos.to_vec2()));
            }
        }
        None
    }
}
