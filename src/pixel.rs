use macroquad::prelude::*;

use crate::model::{Bullet, BulletKind, Enemy, Particle, Treasure, TreasureKind};

type PaletteFn = fn(char) -> Option<Color>;

#[derive(Clone, Copy, Debug)]
pub struct PixelSprite {
    pub w: u8,
    pub h: u8,
    pub rows: &'static [&'static str],
}

impl PixelSprite {
    // 按字符像素点阵绘制精灵
    pub fn draw(&self, pos: Vec2, scale: f32, palette: PaletteFn) {
        let w = self.w as usize;
        let h = self.h as usize;
        if w == 0 || h == 0 || self.rows.is_empty() {
            return;
        }

        // 将精灵中心对齐到目标位置
        let origin = vec2(
            pos.x - self.w as f32 * scale * 0.5,
            pos.y - self.h as f32 * scale * 0.5,
        );

        for (y, row) in self.rows.iter().enumerate().take(h) {
            let bytes = row.as_bytes();
            for x in 0..w {
                let ch = bytes.get(x).copied().unwrap_or(b'.') as char;
                if ch == '.' {
                    continue;
                }
                if let Some(color) = palette(ch) {
                    // 逐像素绘制方块，形成像素风
                    let px = origin.x + x as f32 * scale;
                    let py = origin.y + y as f32 * scale;
                    draw_rectangle(px, py, scale, scale, color);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PixelArt {
    pub sprite: PixelSprite,
    pub palette: PaletteFn,
}

impl PixelArt {
    // 用预设调色板绘制像素精灵
    pub fn draw(&self, pos: Vec2, scale: f32) {
        self.sprite.draw(pos, scale, self.palette);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PixelUi {
    plane_tiers: [PixelArt; 3],
    enemy: PixelArt,
    bullet_normal: PixelArt,
    bullet_spread: PixelArt,
    bullet_laser: PixelArt,
    bullet_enemy: PixelArt,
    treasure: [PixelArt; 6],
    exhaust_tiers: [PixelArt; 3],
    sparkle: PixelArt,
}

impl PixelUi {
    // 初始化像素风UI资源（精灵+调色板）
    pub fn new() -> Self {
        Self {
            plane_tiers: [
                PixelArt {
                    sprite: PLANE_SPRITE,
                    palette: plane_palette_t1,
                },
                PixelArt {
                    sprite: PLANE_SPRITE,
                    palette: plane_palette_t2,
                },
                PixelArt {
                    sprite: PLANE_SPRITE,
                    palette: plane_palette_t3,
                },
            ],
            enemy: PixelArt {
                sprite: ENEMY_SPRITE,
                palette: enemy_palette,
            },
            bullet_normal: PixelArt {
                sprite: BULLET_SPRITE,
                palette: bullet_palette_normal,
            },
            bullet_spread: PixelArt {
                sprite: BULLET_SPREAD_SPRITE,
                palette: bullet_palette_spread,
            },
            bullet_laser: PixelArt {
                sprite: BULLET_LASER_SPRITE,
                palette: bullet_palette_laser,
            },
            bullet_enemy: PixelArt {
                sprite: BULLET_ENEMY_SPRITE,
                palette: bullet_palette_enemy,
            },
            treasure: [
                PixelArt {
                    sprite: TREASURE_STAR,
                    palette: treasure_palette_gold,
                },
                PixelArt {
                    sprite: TREASURE_HEART,
                    palette: treasure_palette_orange,
                },
                PixelArt {
                    sprite: TREASURE_CROSS,
                    palette: treasure_palette_pink,
                },
                PixelArt {
                    sprite: TREASURE_SHIELD,
                    palette: treasure_palette_blue,
                },
                PixelArt {
                    sprite: TREASURE_SPREAD,
                    palette: treasure_palette_green,
                },
                PixelArt {
                    sprite: TREASURE_LASER,
                    palette: treasure_palette_purple,
                },
            ],
            exhaust_tiers: [
                PixelArt {
                    sprite: EXHAUST_T1,
                    palette: exhaust_palette_t1,
                },
                PixelArt {
                    sprite: EXHAUST_T2,
                    palette: exhaust_palette_t2,
                },
                PixelArt {
                    sprite: EXHAUST_T3,
                    palette: exhaust_palette_t3,
                },
            ],
            sparkle: PixelArt {
                sprite: SPARKLE_SPRITE,
                palette: sparkle_palette,
            },
        }
    }

    // 将等级映射为视觉档位
    fn tier_from_level(level: usize) -> usize {
        match level {
            0..=4 => 0,
            5..=9 => 1,
            _ => 2,
        }
    }

    // 绘制玩家飞机与尾焰
    pub fn draw_plane(&self, level: usize, pos: Vec2, size: Vec2, boosted: bool) {
        let tier = Self::tier_from_level(level);
        let plane = &self.plane_tiers[tier];
        let scale = (size.x / plane.sprite.w as f32).min(size.y / plane.sprite.h as f32);
        if boosted {
            let exhaust = &self.exhaust_tiers[tier];
            let ex_scale = scale * 0.7;
            let ex_offset = vec2(0.0, size.y * 0.45);
            // 双喷口效果
            exhaust.draw(pos + ex_offset + vec2(-size.x * 0.18, 0.0), ex_scale);
            exhaust.draw(pos + ex_offset + vec2(size.x * 0.18, 0.0), ex_scale);
        }
        plane.draw(pos, scale);
    }

    // 绘制等级特效（环绕光点）
    pub fn draw_level_effect(&self, level: usize, pos: Vec2, size: Vec2) {
        let tier = Self::tier_from_level(level);
        if tier == 0 {
            return;
        }
        let t = get_time() as f32;
        let radius = size.x * (0.45 + tier as f32 * 0.08);
        for i in 0..(2 + tier) {
            let angle = t * 2.2 + i as f32 * 2.3;
            let sparkle_pos = pos + vec2(angle.cos(), angle.sin()) * radius;
            let scale = (size.x * 0.08).max(1.5);
            self.sparkle.draw(sparkle_pos, scale);
        }
    }

    // 绘制无敌光环
    pub fn draw_invincible_aura(&self, pos: Vec2, size: Vec2) {
        let t = get_time() as f32;
        let radius = size.x * 0.6;
        for i in 0..4 {
            let angle = t * 3.0 + i as f32 * 1.6;
            let sparkle_pos = pos + vec2(angle.cos(), angle.sin()) * radius;
            let scale = (size.x * 0.09).max(1.5);
            self.sparkle.draw(sparkle_pos, scale);
        }
    }

    // 绘制敌机
    pub fn draw_enemy(&self, enemy: &Enemy) {
        let scale = (enemy.size.x / self.enemy.sprite.w as f32)
            .min(enemy.size.y / self.enemy.sprite.h as f32);
        self.enemy.draw(enemy.pos, scale);
        if enemy.hp > 1 {
            let ring_scale = scale * 0.9;
            self.sparkle.draw(enemy.pos + vec2(0.0, -enemy.size.y * 0.25), ring_scale);
        }
    }

    // 绘制子弹
    pub fn draw_bullet(&self, bullet: &Bullet) {
        let art = match bullet.kind {
            BulletKind::PlayerNormal => &self.bullet_normal,
            BulletKind::PlayerSpread => &self.bullet_spread,
            BulletKind::PlayerLaser => &self.bullet_laser,
            BulletKind::Enemy => &self.bullet_enemy,
        };
        let size = bullet.radius * 2.0;
        let scale = (size / art.sprite.w as f32).max(1.2);
        art.draw(bullet.pos, scale);
    }

    // 绘制宝物
    pub fn draw_treasure(&self, treasure: &Treasure) {
        let idx = match treasure.kind {
            TreasureKind::BulletUpgradePermanent => 0,
            TreasureKind::MaxLifePermanent => 1,
            TreasureKind::LifePlus => 2,
            TreasureKind::InvincibleTimed => 3,
            TreasureKind::SpreadTimed => 4,
            TreasureKind::LaserTimed => 5,
        };
        let art = &self.treasure[idx];
        let size = treasure.radius * 2.0;
        let scale = (size / art.sprite.w as f32).max(1.5);
        art.draw(treasure.pos, scale);
    }

    // 绘制粒子效果（带透明度衰减）
    pub fn draw_particle(&self, particle: &Particle) {
        let size = (particle.radius * 2.0).max(1.5);
        let scale = size / self.sparkle.sprite.w as f32;
        let alpha = (particle.life / 0.45).clamp(0.0, 1.0);
        let color = Color::new(particle.color.r, particle.color.g, particle.color.b, alpha);
        let origin = vec2(
            particle.pos.x - self.sparkle.sprite.w as f32 * scale * 0.5,
            particle.pos.y - self.sparkle.sprite.h as f32 * scale * 0.5,
        );
        for (y, row) in self.sparkle.sprite.rows.iter().enumerate() {
            let bytes = row.as_bytes();
            for x in 0..self.sparkle.sprite.w as usize {
                let ch = bytes.get(x).copied().unwrap_or(b'.') as char;
                if ch == '.' {
                    continue;
                }
                let px = origin.x + x as f32 * scale;
                let py = origin.y + y as f32 * scale;
                draw_rectangle(px, py, scale, scale, color);
            }
        }
    }
}

const PLANE_SPRITE: PixelSprite = PixelSprite {
    w: 11,
    h: 13,
    rows: &[
        ".....P.....",
        "....PPP....",
        "...PPPPP...",
        "..PPWWWPP..",
        ".PPWWWWWPP.",
        ".PPWWCWWPP.",
        ".PPWWWWWPP.",
        "..PPWWWPP..",
        "...PPPPP...",
        "....PPP....",
        "...P...P...",
        "...F...F...",
        "..FFF.FFF..",
    ],
};

const ENEMY_SPRITE: PixelSprite = PixelSprite {
    w: 9,
    h: 7,
    rows: &[
        "..EEE....",
        ".EEEEEE..",
        ".EEOEEO..",
        ".EEEEEE..",
        "..EE.EE..",
        "..E....E.",
        "...EEEE..",
    ],
};

const BULLET_SPRITE: PixelSprite = PixelSprite {
    w: 3,
    h: 5,
    rows: &[".B.", "BBB", "BBB", "BBB", ".B."],
};

const BULLET_SPREAD_SPRITE: PixelSprite = PixelSprite {
    w: 3,
    h: 5,
    rows: &[".S.", "SSS", "SSS", "SSS", ".S."],
};

const BULLET_LASER_SPRITE: PixelSprite = PixelSprite {
    w: 5,
    h: 11,
    rows: &[
        "..L..",
        ".LLL.",
        "LLLLL",
        "LLLLL",
        "LLLLL",
        "LLLLL",
        "LLLLL",
        "LLLLL",
        "LLLLL",
        ".LLL.",
        "..L..",
    ],
};

const BULLET_ENEMY_SPRITE: PixelSprite = PixelSprite {
    w: 3,
    h: 5,
    rows: &[".E.", "EEE", "EEE", "EEE", ".E."],
};

const TREASURE_STAR: PixelSprite = PixelSprite {
    w: 7,
    h: 7,
    rows: &[
        "..A.A..",
        ".AAAAA.",
        "AAAAAAA",
        ".AAAAA.",
        "..AAA..",
        "...A...",
        "..A.A..",
    ],
};

const TREASURE_HEART: PixelSprite = PixelSprite {
    w: 7,
    h: 7,
    rows: &[
        ".HH.HH.",
        "HHHHHHH",
        "HHHHHHH",
        ".HHHHH.",
        "..HHH..",
        "...H...",
        ".......",
    ],
};

const TREASURE_CROSS: PixelSprite = PixelSprite {
    w: 7,
    h: 7,
    rows: &[
        "...P...",
        "...P...",
        "..PPP..",
        ".PPPPP.",
        "..PPP..",
        "...P...",
        "...P...",
    ],
};

const TREASURE_SHIELD: PixelSprite = PixelSprite {
    w: 7,
    h: 7,
    rows: &[
        "..SSS..",
        ".SSSSS.",
        ".SS.SS.",
        ".SSSSS.",
        "..SSS..",
        "..S.S..",
        ".......",
    ],
};

const TREASURE_SPREAD: PixelSprite = PixelSprite {
    w: 7,
    h: 7,
    rows: &[
        "..V.V..",
        ".VVVVV.",
        "VVVVVVV",
        "..V.V..",
        ".V...V.",
        "V.....V",
        "..V.V..",
    ],
};

const TREASURE_LASER: PixelSprite = PixelSprite {
    w: 7,
    h: 7,
    rows: &[
        "...L...",
        "..LLL..",
        ".LLLLL.",
        "LLL.LLL",
        ".LLLLL.",
        "..LLL..",
        "...L...",
    ],
};

const EXHAUST_T1: PixelSprite = PixelSprite {
    w: 5,
    h: 5,
    rows: &["..F..", ".FFF.", "FFFFF", ".FFF.", "..F.."],
};

const EXHAUST_T2: PixelSprite = PixelSprite {
    w: 5,
    h: 5,
    rows: &["..G..", ".GGG.", "GGGGG", ".GGG.", "..G.."],
};

const EXHAUST_T3: PixelSprite = PixelSprite {
    w: 5,
    h: 5,
    rows: &["..H..", ".HHH.", "HHHHH", ".HHH.", "..H.."],
};

const SPARKLE_SPRITE: PixelSprite = PixelSprite {
    w: 3,
    h: 3,
    rows: &[".*.", "***", ".*."],
};

// 飞机一阶配色
fn plane_palette_t1(ch: char) -> Option<Color> {
    match ch {
        'P' => Some(Color::new(0.2, 0.85, 0.4, 1.0)),
        'W' => Some(Color::new(0.12, 0.65, 0.3, 1.0)),
        'C' => Some(Color::new(0.55, 0.85, 1.0, 1.0)),
        'F' => Some(Color::new(1.0, 0.6, 0.2, 1.0)),
        _ => None,
    }
}

// 飞机二阶配色
fn plane_palette_t2(ch: char) -> Option<Color> {
    match ch {
        'P' => Some(Color::new(0.2, 0.75, 0.95, 1.0)),
        'W' => Some(Color::new(0.1, 0.55, 0.85, 1.0)),
        'C' => Some(Color::new(0.8, 0.95, 1.0, 1.0)),
        'F' => Some(Color::new(1.0, 0.7, 0.35, 1.0)),
        _ => None,
    }
}

// 飞机三阶配色
fn plane_palette_t3(ch: char) -> Option<Color> {
    match ch {
        'P' => Some(Color::new(0.95, 0.35, 0.65, 1.0)),
        'W' => Some(Color::new(0.75, 0.2, 0.55, 1.0)),
        'C' => Some(Color::new(1.0, 0.85, 0.95, 1.0)),
        'F' => Some(Color::new(1.0, 0.8, 0.4, 1.0)),
        _ => None,
    }
}

// 敌机配色
fn enemy_palette(ch: char) -> Option<Color> {
    match ch {
        'E' => Some(Color::new(0.9, 0.2, 0.2, 1.0)),
        'O' => Some(Color::new(1.0, 0.9, 0.3, 1.0)),
        _ => None,
    }
}

// 普通子弹配色
fn bullet_palette_normal(ch: char) -> Option<Color> {
    match ch {
        'B' => Some(Color::new(1.0, 0.9, 0.2, 1.0)),
        _ => None,
    }
}

// 散射子弹配色
fn bullet_palette_spread(ch: char) -> Option<Color> {
    match ch {
        'S' => Some(Color::new(0.2, 1.0, 0.5, 1.0)),
        _ => None,
    }
}

// 激光子弹配色
fn bullet_palette_laser(ch: char) -> Option<Color> {
    match ch {
        'L' => Some(Color::new(0.8, 0.4, 1.0, 1.0)),
        _ => None,
    }
}

// 敌机子弹配色
fn bullet_palette_enemy(ch: char) -> Option<Color> {
    match ch {
        'E' => Some(Color::new(0.7, 0.7, 0.7, 1.0)),
        _ => None,
    }
}

// 金色宝物配色
fn treasure_palette_gold(ch: char) -> Option<Color> {
    match ch {
        'A' => Some(GOLD),
        _ => None,
    }
}

// 橙色宝物配色
fn treasure_palette_orange(ch: char) -> Option<Color> {
    match ch {
        'H' => Some(ORANGE),
        _ => None,
    }
}

// 粉色宝物配色
fn treasure_palette_pink(ch: char) -> Option<Color> {
    match ch {
        'P' => Some(PINK),
        _ => None,
    }
}

// 蓝色宝物配色
fn treasure_palette_blue(ch: char) -> Option<Color> {
    match ch {
        'S' => Some(SKYBLUE),
        _ => None,
    }
}

// 绿色宝物配色
fn treasure_palette_green(ch: char) -> Option<Color> {
    match ch {
        'V' => Some(GREEN),
        _ => None,
    }
}

// 紫色宝物配色
fn treasure_palette_purple(ch: char) -> Option<Color> {
    match ch {
        'L' => Some(PURPLE),
        _ => None,
    }
}

// 尾焰一阶配色
fn exhaust_palette_t1(ch: char) -> Option<Color> {
    match ch {
        'F' => Some(Color::new(1.0, 0.6, 0.2, 0.9)),
        _ => None,
    }
}

// 尾焰二阶配色
fn exhaust_palette_t2(ch: char) -> Option<Color> {
    match ch {
        'G' => Some(Color::new(1.0, 0.8, 0.3, 0.95)),
        _ => None,
    }
}

// 尾焰三阶配色
fn exhaust_palette_t3(ch: char) -> Option<Color> {
    match ch {
        'H' => Some(Color::new(1.0, 0.95, 0.5, 1.0)),
        _ => None,
    }
}

// 闪光粒子配色
fn sparkle_palette(ch: char) -> Option<Color> {
    match ch {
        '*' => Some(Color::new(0.7, 0.95, 1.0, 1.0)),
        _ => None,
    }
}
