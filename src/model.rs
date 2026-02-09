use macroquad::prelude::*;

use crate::config::{PLANE_LEVELS, SCORE_PER_LEVEL, SCREEN_H, SCREEN_W};
use crate::save::PlayerProfile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppMode {
    Splash,
    EnterName,
    Menu,
    Leaderboard,
    Settings,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulletMode {
    Normal,
    Double,
    Triple,
    Spread,
    Laser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulletKind {
    PlayerNormal,
    PlayerSpread,
    PlayerLaser,
    Enemy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreasureKind {
    BulletUpgradePermanent,
    MaxLifePermanent,
    LifePlus,
    InvincibleTimed,
    SpreadTimed,
    LaserTimed,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: Vec2,
    pub size: Vec2,
    pub lives: i32,
    pub max_lives: i32,
    pub speed: f32,
    pub invincible_until: f64,
    pub shot_cooldown: f32,
    pub shot_timer: f32,
    pub base_bullet_level: u8,
    pub manual_mode: Option<BulletMode>,
    pub temp_mode: Option<(BulletMode, f64)>,
}

impl Player {
    // 获取玩家碰撞矩形
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x - self.size.x * 0.5,
            self.pos.y - self.size.y * 0.5,
            self.size.x,
            self.size.y,
        )
    }

    // 判断是否处于无敌时间内
    pub fn is_invincible(&self) -> bool {
        get_time() < self.invincible_until
    }

    // 计算当前子弹模式（临时效果优先，其次手动选择，最后基础等级）
    pub fn bullet_mode(&self) -> BulletMode {
        // 临时加成优先
        if let Some((mode, until)) = self.temp_mode {
            if get_time() <= until {
                return mode;
            }
        }
        // 手动选择优先于默认等级
        if let Some(mode) = self.manual_mode {
            return mode;
        }
        match self.base_bullet_level {
            1 => BulletMode::Normal,
            2 => BulletMode::Double,
            _ => BulletMode::Triple,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub pos: Vec2,
    pub size: Vec2,
    pub vel: Vec2,
    pub hp: i32,
    pub shot_timer: f32,
}

impl Enemy {
    // 获取敌机碰撞矩形
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x - self.size.x * 0.5,
            self.pos.y - self.size.y * 0.5,
            self.size.x,
            self.size.y,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub damage: i32,
    pub from_player: bool,
    pub kind: BulletKind,
}

#[derive(Clone, Debug)]
pub struct Treasure {
    pub pos: Vec2,
    pub vel: Vec2,
    pub kind: TreasureKind,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub player: Player,
    pub bullets: Vec<Bullet>,
    pub enemies: Vec<Enemy>,
    pub treasures: Vec<Treasure>,
    pub particles: Vec<Particle>,
    pub score: u32,
    pub enemy_spawn_timer: f32,
    pub enemy_spawn_interval: f32,
    pub game_over_cooldown: f32,
    pub just_saved_score: bool,
    pub auto_fire: bool,
}

impl Game {
    // 根据玩家档案初始化一局新游戏
    pub fn new(profile: &PlayerProfile) -> Self {
        let max_lives = profile.permanent.max_lives.max(1) as i32;
        let player = Player {
            pos: vec2(SCREEN_W * 0.5, SCREEN_H - 60.0),
            size: vec2(32.0, 40.0),
            lives: max_lives,
            max_lives,
            speed: 320.0,
            invincible_until: 0.0,
            shot_cooldown: 0.14,
            shot_timer: 0.0,
            base_bullet_level: profile.permanent.bullet_level.max(1).min(3),
            manual_mode: None,
            temp_mode: None,
        };
        Self {
            player,
            bullets: Vec::new(),
            enemies: Vec::new(),
            treasures: Vec::new(),
            particles: Vec::new(),
            score: 0,
            enemy_spawn_timer: 0.0,
            enemy_spawn_interval: 0.85,
            game_over_cooldown: 0.0,
            just_saved_score: false,
            auto_fire: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub life: f32,
    pub color: Color,
}

// 根据分数换算飞机等级
pub fn plane_level_from_score(score: u32) -> usize {
    let idx = (score / SCORE_PER_LEVEL) as usize;
    idx.min(PLANE_LEVELS.saturating_sub(1))
}

// 根据等级索引返回境界名称
pub fn plane_level_name(level: usize) -> &'static str {
    const NAMES: [&str; PLANE_LEVELS] = [
        "炼气期·初期",
        "炼气期·中期",
        "炼气期·后期",
        "筑基期·初期",
        "筑基期·中期",
        "筑基期·后期",
        "结丹期·初期",
        "结丹期·中期",
        "结丹期·后期",
        "元婴期·初期",
        "元婴期·中期",
        "元婴期·后期",
        "化神期·初期",
        "化神期·中期",
        "化神期·后期",
    ];
    NAMES[level.min(PLANE_LEVELS - 1)]
}
