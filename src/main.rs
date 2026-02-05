mod save;

use std::io;
use std::path::Path;

use macroquad::prelude::*;

use crate::save::{Leaderboard, PlayerProfile, SaveStore, ScoreEntry};

const SCREEN_W: f32 = 480.0;
const SCREEN_H: f32 = 720.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "飞机大战 (Rust)".to_string(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        high_dpi: true,
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppMode {
    EnterName,
    Menu,
    Leaderboard,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BulletMode {
    Normal,
    Double,
    Triple,
    Spread,
    Laser,
}

const PLANE_FRAMES: usize = 8;
const PLANE_LEVELS: usize = 15;
const SCORE_PER_LEVEL: u32 = 100;

#[derive(Clone, Debug)]
struct Player {
    pos: Vec2,
    size: Vec2,
    lives: i32,
    max_lives: i32,
    speed: f32,
    invincible_until: f64,
    shot_cooldown: f32,
    shot_timer: f32,
    base_bullet_level: u8,
    manual_mode: Option<BulletMode>,
    temp_mode: Option<(BulletMode, f64)>,
}

impl Player {
    fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x - self.size.x * 0.5,
            self.pos.y - self.size.y * 0.5,
            self.size.x,
            self.size.y,
        )
    }

    fn is_invincible(&self) -> bool {
        get_time() < self.invincible_until
    }

    fn bullet_mode(&self) -> BulletMode {
        if let Some((mode, until)) = self.temp_mode {
            if get_time() <= until {
                return mode;
            }
        }
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
struct Enemy {
    pos: Vec2,
    size: Vec2,
    vel: Vec2,
    hp: i32,
    shot_timer: f32,
}

impl Enemy {
    fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x - self.size.x * 0.5,
            self.pos.y - self.size.y * 0.5,
            self.size.x,
            self.size.y,
        )
    }
}

#[derive(Clone, Debug)]
struct Bullet {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    damage: i32,
    from_player: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreasureKind {
    BulletUpgradePermanent,
    MaxLifePermanent,
    LifePlus,
    InvincibleTimed,
    SpreadTimed,
    LaserTimed,
}

#[derive(Clone, Debug)]
struct Treasure {
    pos: Vec2,
    vel: Vec2,
    kind: TreasureKind,
    radius: f32,
}

impl Treasure {
    fn label(&self) -> &'static str {
        match self.kind {
            TreasureKind::BulletUpgradePermanent => "B+",
            TreasureKind::MaxLifePermanent => "HP+",
            TreasureKind::LifePlus => "HP",
            TreasureKind::InvincibleTimed => "I",
            TreasureKind::SpreadTimed => "S",
            TreasureKind::LaserTimed => "L",
        }
    }

    fn color(&self) -> Color {
        match self.kind {
            TreasureKind::BulletUpgradePermanent => GOLD,
            TreasureKind::MaxLifePermanent => ORANGE,
            TreasureKind::LifePlus => PINK,
            TreasureKind::InvincibleTimed => SKYBLUE,
            TreasureKind::SpreadTimed => GREEN,
            TreasureKind::LaserTimed => PURPLE,
        }
    }
}

#[derive(Clone, Debug)]
struct Game {
    player: Player,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    treasures: Vec<Treasure>,
    particles: Vec<Particle>,
    score: u32,
    enemy_spawn_timer: f32,
    enemy_spawn_interval: f32,
    game_over_cooldown: f32,
    just_saved_score: bool,
    auto_fire: bool,
}

impl Game {
    fn new(profile: &PlayerProfile) -> Self {
        let max_lives = profile.permanent.max_lives.max(1) as i32;
        let player = Player {
            pos: vec2(SCREEN_W * 0.5, SCREEN_H - 70.0),
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
struct Particle {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    life: f32,
    color: Color,
}

#[derive(Default)]
struct NameInput {
    buffer: String,
    error: Option<(String, f64)>,
}

struct Ui {
    font: Option<Font>,
    plane_sheet: Option<PlaneSheet>,
}

impl Ui {
    fn font(&self) -> Option<&Font> {
        self.font.as_ref()
    }
}

#[derive(Clone, Debug)]
struct PlaneSheet {
    texture: Texture2D,
    cols: usize,
    rows: usize,
    frame_w: f32,
    frame_h: f32,
}

impl PlaneSheet {
    fn frame(&self, level: usize, frame: usize) -> Rect {
        let col = frame % self.cols;
        let row = level % self.rows;
        Rect::new(
            col as f32 * self.frame_w,
            row as f32 * self.frame_h,
            self.frame_w,
            self.frame_h,
        )
    }
}

fn clamp_vec2(v: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    vec2(v.x.clamp(min.x, max.x), v.y.clamp(min.y, max.y))
}

fn plane_anim_frame() -> usize {
    let fps = 12.0;
    ((get_time() * fps) as usize) % PLANE_FRAMES
}

fn plane_level_from_score(score: u32) -> usize {
    let idx = (score / SCORE_PER_LEVEL) as usize;
    idx.min(PLANE_LEVELS.saturating_sub(1))
}

fn plane_level_name(level: usize) -> &'static str {
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

fn draw_text_ui(ui: &Ui, text: &str, x: f32, y: f32, size: u16, color: Color) {
    if let Some(font) = ui.font() {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: size,
                color,
                ..Default::default()
            },
        );
    } else {
        draw_text(text, x, y, size as f32, color);
    }
}

fn measure_text_ui(ui: &Ui, text: &str, size: u16) -> TextDimensions {
    measure_text(text, ui.font(), size, 1.0)
}

fn draw_centered_text(ui: &Ui, text: &str, y: f32, size: u16, color: Color) {
    let dims = measure_text_ui(ui, text, size);
    draw_text_ui(ui, text, (SCREEN_W - dims.width) * 0.5, y, size, color);
}

fn circle_hits_rect(center: Vec2, radius: f32, rect: Rect) -> bool {
    let cx = center.x.clamp(rect.x, rect.x + rect.w);
    let cy = center.y.clamp(rect.y, rect.y + rect.h);
    let dx = center.x - cx;
    let dy = center.y - cy;
    dx * dx + dy * dy <= radius * radius
}

fn record_score(lb: &mut Leaderboard, username: &str, score: u32) {
    lb.entries.push(ScoreEntry {
        username: username.to_string(),
        score,
    });
    lb.entries.sort_by(|a, b| b.score.cmp(&a.score));
    lb.entries.truncate(20);
}

fn apply_treasure(
    store: &SaveStore,
    profile: &mut PlayerProfile,
    player: &mut Player,
    kind: TreasureKind,
) -> io::Result<()> {
    match kind {
        TreasureKind::BulletUpgradePermanent => {
            profile.permanent.bullet_level = (profile.permanent.bullet_level + 1).min(3);
            player.base_bullet_level = profile.permanent.bullet_level;
            store.save_profile(profile)?;
        }
        TreasureKind::MaxLifePermanent => {
            profile.permanent.max_lives = (profile.permanent.max_lives + 1).min(9);
            let new_max = profile.permanent.max_lives as i32;
            player.max_lives = new_max;
            player.lives = (player.lives + 1).min(new_max);
            store.save_profile(profile)?;
        }
        TreasureKind::LifePlus => {
            player.lives = (player.lives + 1).min(player.max_lives);
        }
        TreasureKind::InvincibleTimed => {
            player.invincible_until = get_time() + 5.0;
        }
        TreasureKind::SpreadTimed => {
            player.temp_mode = Some((BulletMode::Spread, get_time() + 9.0));
        }
        TreasureKind::LaserTimed => {
            player.temp_mode = Some((BulletMode::Laser, get_time() + 6.0));
        }
    }
    Ok(())
}

fn spawn_enemy(game: &mut Game) {
    let x = rand::gen_range(40.0, SCREEN_W - 40.0);
    let speed = rand::gen_range(90.0, 150.0);
    let hp = if rand::gen_range(0, 10) == 0 { 2 } else { 1 };
    game.enemies.push(Enemy {
        pos: vec2(x, -20.0),
        size: vec2(34.0, 28.0),
        vel: vec2(rand::gen_range(-20.0, 20.0), speed),
        hp,
        shot_timer: rand::gen_range(0.6, 1.4),
    });
}

fn maybe_drop_treasure(treasures: &mut Vec<Treasure>, at: Vec2) {
    let roll = rand::gen_range(0, 100);
    if roll >= 30 {
        return;
    }

    let kind = if roll < 3 {
        TreasureKind::MaxLifePermanent
    } else if roll < 8 {
        TreasureKind::BulletUpgradePermanent
    } else if roll < 14 {
        TreasureKind::InvincibleTimed
    } else if roll < 21 {
        TreasureKind::LaserTimed
    } else if roll < 27 {
        TreasureKind::SpreadTimed
    } else {
        TreasureKind::LifePlus
    };

    treasures.push(Treasure {
        pos: at,
        vel: vec2(0.0, rand::gen_range(90.0, 150.0)),
        kind,
        radius: 12.0,
    });
}

fn shoot_player(game: &mut Game) {
    let mode = game.player.bullet_mode();
    let base_speed = 520.0;
    let p = game.player.pos;
    match mode {
        BulletMode::Normal => {
            game.bullets.push(Bullet {
                pos: p + vec2(0.0, -26.0),
                vel: vec2(0.0, -base_speed),
                radius: 4.0,
                damage: 1,
                from_player: true,
            });
        }
        BulletMode::Double => {
            for dx in [-9.0, 9.0] {
                game.bullets.push(Bullet {
                    pos: p + vec2(dx, -26.0),
                    vel: vec2(0.0, -base_speed),
                    radius: 4.0,
                    damage: 1,
                    from_player: true,
                });
            }
        }
        BulletMode::Triple => {
            for dx in [-12.0, 0.0, 12.0] {
                game.bullets.push(Bullet {
                    pos: p + vec2(dx, -26.0),
                    vel: vec2(0.0, -base_speed),
                    radius: 4.0,
                    damage: 1,
                    from_player: true,
                });
            }
        }
        BulletMode::Spread => {
            for (dx, vx) in [(-10.0, -120.0), (0.0, 0.0), (10.0, 120.0)] {
                game.bullets.push(Bullet {
                    pos: p + vec2(dx, -26.0),
                    vel: vec2(vx, -base_speed),
                    radius: 4.0,
                    damage: 1,
                    from_player: true,
                });
            }
        }
        BulletMode::Laser => {
            game.bullets.push(Bullet {
                pos: p + vec2(0.0, -30.0),
                vel: vec2(0.0, -base_speed * 1.2),
                radius: 7.0,
                damage: 3,
                from_player: true,
            });
        }
    }
}

fn shoot_enemy(bullets: &mut Vec<Bullet>, enemy_pos: Vec2) {
    bullets.push(Bullet {
        pos: enemy_pos + vec2(0.0, 14.0),
        vel: vec2(0.0, rand::gen_range(240.0, 320.0)),
        radius: 4.0,
        damage: 1,
        from_player: false,
    });
}

fn update_playing(
    store: &SaveStore,
    profile: &mut PlayerProfile,
    leaderboard: &mut Leaderboard,
    game: &mut Game,
) -> io::Result<bool> {
    let dt = get_frame_time();

    if is_key_pressed(KeyCode::F) {
        game.auto_fire = !game.auto_fire;
    }
    if is_key_pressed(KeyCode::Key1) {
        game.player.manual_mode = Some(BulletMode::Normal);
    }
    if is_key_pressed(KeyCode::Key2) {
        game.player.manual_mode = Some(BulletMode::Double);
    }
    if is_key_pressed(KeyCode::Key3) {
        game.player.manual_mode = Some(BulletMode::Triple);
    }
    if is_key_pressed(KeyCode::Key4) {
        game.player.manual_mode = Some(BulletMode::Spread);
    }
    if is_key_pressed(KeyCode::Key5) {
        game.player.manual_mode = Some(BulletMode::Laser);
    }
    if is_key_pressed(KeyCode::Key0) {
        game.player.manual_mode = None;
    }
    if is_key_pressed(KeyCode::I) {
        game.player.invincible_until = get_time() + 6.0;
    }

    if let Some((_, until)) = game.player.temp_mode {
        if get_time() > until {
            game.player.temp_mode = None;
        }
    }

    let mut move_dir = vec2(0.0, 0.0);
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        move_dir.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        move_dir.x += 1.0;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        move_dir.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        move_dir.y += 1.0;
    }
    if move_dir.length_squared() > 0.0 {
        move_dir = move_dir.normalize();
    }
    game.player.pos += move_dir * game.player.speed * dt;
    game.player.pos = clamp_vec2(
        game.player.pos,
        vec2(24.0, 40.0),
        vec2(SCREEN_W - 24.0, SCREEN_H - 26.0),
    );

    game.player.shot_timer = (game.player.shot_timer - dt).max(0.0);
    if (game.auto_fire || is_key_down(KeyCode::Space) || is_mouse_button_down(MouseButton::Left))
        && game.player.shot_timer <= 0.0
    {
        shoot_player(game);
        game.player.shot_timer = game.player.shot_cooldown;
    }

    game.enemy_spawn_timer += dt;
    if game.enemy_spawn_timer >= game.enemy_spawn_interval {
        game.enemy_spawn_timer = 0.0;
        spawn_enemy(game);
        if game.enemy_spawn_interval > 0.35 {
            game.enemy_spawn_interval *= 0.995;
        }
    }

    {
        let (enemies, bullets) = (&mut game.enemies, &mut game.bullets);
        for enemy in enemies {
            enemy.pos += enemy.vel * dt;
            enemy.pos.x = enemy.pos.x.clamp(20.0, SCREEN_W - 20.0);
            enemy.shot_timer -= dt;
            if enemy.shot_timer <= 0.0 && enemy.pos.y > 20.0 && enemy.pos.y < SCREEN_H * 0.85 {
                shoot_enemy(bullets, enemy.pos);
                enemy.shot_timer = rand::gen_range(0.9, 1.6);
            }
        }
    }

    for bullet in &mut game.bullets {
        bullet.pos += bullet.vel * dt;
    }

    for t in &mut game.treasures {
        t.pos += t.vel * dt;
    }

    if game.player.is_invincible() {
        let base = game.player.pos;
        for _ in 0..3 {
            let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            let radius = rand::gen_range(16.0, 26.0);
            let pos = base + vec2(angle.cos(), angle.sin()) * radius;
            let vel = vec2(rand::gen_range(-20.0, 20.0), rand::gen_range(-20.0, 20.0));
            game.particles.push(Particle {
                pos,
                vel,
                radius: rand::gen_range(1.0, 2.4),
                life: rand::gen_range(0.2, 0.45),
                color: SKYBLUE,
            });
        }
    }

    for p in &mut game.particles {
        p.life -= dt;
        p.pos += p.vel * dt;
    }
    game.particles.retain(|p| p.life > 0.0);

    let player_rect = game.player.rect();

    let mut player_hit = false;

    let mut bullet_alive = vec![true; game.bullets.len()];
    for bi in 0..game.bullets.len() {
        if !bullet_alive[bi] {
            continue;
        }

        let (pos, radius, damage, from_player) = {
            let b = &game.bullets[bi];
            (b.pos, b.radius, b.damage, b.from_player)
        };

        if from_player {
            for ei in 0..game.enemies.len() {
                let er = game.enemies[ei].rect();
                if circle_hits_rect(pos, radius, er) {
                    game.enemies[ei].hp -= damage;
                    bullet_alive[bi] = false;
                    break;
                }
            }
        } else if circle_hits_rect(pos, radius, player_rect) {
            player_hit = true;
            bullet_alive[bi] = false;
        }
    }

    let mut remaining_bullets = Vec::with_capacity(game.bullets.len());
    for (bi, bullet) in game.bullets.drain(..).enumerate() {
        if !bullet_alive.get(bi).copied().unwrap_or(false) {
            continue;
        }
        if bullet.pos.y < -40.0
            || bullet.pos.y > SCREEN_H + 40.0
            || bullet.pos.x < -40.0
            || bullet.pos.x > SCREEN_W + 40.0
        {
            continue;
        }
        remaining_bullets.push(bullet);
    }
    game.bullets = remaining_bullets;

    let mut keep_enemies = Vec::with_capacity(game.enemies.len());
    {
        let treasures = &mut game.treasures;
        for mut enemy in game.enemies.drain(..) {
            if enemy.pos.y > SCREEN_H + 50.0 {
                continue;
            }

            if enemy.rect().overlaps(&player_rect) {
                enemy.hp = 0;
                player_hit = true;
            }

            if enemy.hp <= 0 {
                game.score = game.score.saturating_add(10);
                maybe_drop_treasure(treasures, enemy.pos);
                continue;
            }

            keep_enemies.push(enemy);
        }
    }
    game.enemies = keep_enemies;

    let mut keep_treasures = Vec::with_capacity(game.treasures.len());
    let mut score_bonus = 0u32;
    {
        let player = &mut game.player;
        for t in game.treasures.drain(..) {
            if t.pos.y > SCREEN_H + 50.0 {
                continue;
            }
            let dist2 = (t.pos - player.pos).length_squared();
            if dist2 <= (t.radius + 18.0) * (t.radius + 18.0) {
                apply_treasure(store, profile, player, t.kind)?;
                score_bonus = score_bonus.saturating_add(2);
                continue;
            }
            keep_treasures.push(t);
        }
    }
    game.treasures = keep_treasures;
    game.score = game.score.saturating_add(score_bonus);

    if player_hit && !game.player.is_invincible() {
        game.player.lives -= 1;
        game.player.invincible_until = get_time() + 0.9;
        if game.player.lives <= 0 {
            if !game.just_saved_score {
                record_score(leaderboard, &profile.username, game.score);
                store.save_leaderboard(leaderboard)?;
                game.just_saved_score = true;
            }
            return Ok(true);
        }
    }

    Ok(false)
}

fn draw_playing(ui: &Ui, profile: &PlayerProfile, game: &Game) {
    clear_background(BLACK);

    let level = plane_level_from_score(game.score);
    let level_name = plane_level_name(level);
    let hud = format!(
        "{}  Score: {}  Lives: {}/{}  {}",
        profile.username, game.score, game.player.lives, game.player.max_lives, level_name
    );
    draw_text_ui(ui, &hud, 12.0, 26.0, 24, WHITE);

    if game.auto_fire {
        draw_text_ui(ui, "AUTO FIRE", 12.0, 50.0, 20, SKYBLUE);
    }
    if game.player.is_invincible() {
        let y = if game.auto_fire { 72.0 } else { 50.0 };
        draw_text_ui(ui, "INVINCIBLE", 12.0, y, 20, SKYBLUE);
    }
    if let Some((mode, until)) = game.player.temp_mode {
        let remain = (until - get_time()).max(0.0);
        let label = match mode {
            BulletMode::Spread => "SPREAD",
            BulletMode::Laser => "LASER",
            _ => "POWER",
        };
        let y = match (game.auto_fire, game.player.is_invincible()) {
            (true, true) => 94.0,
            (true, false) | (false, true) => 72.0,
            (false, false) => 50.0,
        };
        draw_text_ui(
            ui,
            &format!("{label} {:.1}s", remain),
            12.0,
            y,
            20,
            GREEN,
        );
    }

    let frame = plane_anim_frame();
    let player_color = if game.player.is_invincible() { SKYBLUE } else { LIME };
    if let Some(sheet) = &ui.plane_sheet {
        draw_plane_sprite(
            sheet,
            level,
            frame,
            game.player.pos,
            game.player.size,
            game.player.is_invincible(),
        );
    } else {
        draw_plane(
            game.player.pos,
            game.player.size,
            player_color,
            game.player.is_invincible(),
        );
    }
    if game.player.is_invincible() {
        let base = game.player.pos;
        let t = get_time() as f32;
        for i in 0..3 {
            let phase = t * 3.0 + i as f32 * 2.1;
            let r1 = 26.0 + phase.sin() * 4.0;
            let r2 = 12.0 + phase.cos() * 3.0;
            let a1 = phase;
            let a2 = phase + 1.4;
            let p1 = base + vec2(a1.cos(), a1.sin()) * r1;
            let p2 = base + vec2(a2.cos(), a2.sin()) * r2;
            let mid = (p1 + p2) * 0.5 + vec2((t * 12.0).sin(), (t * 9.0).cos()) * 4.0;
            draw_line(p1.x, p1.y, mid.x, mid.y, 2.0, SKYBLUE);
            draw_line(mid.x, mid.y, p2.x, p2.y, 1.5, WHITE);
        }
    }

    for enemy in &game.enemies {
        let r = enemy.rect();
        draw_rectangle(r.x, r.y, r.w, r.h, RED);
        if enemy.hp > 1 {
            draw_rectangle_lines(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, 2.0, ORANGE);
        }
    }

    for bullet in &game.bullets {
        let color = if bullet.from_player { YELLOW } else { GRAY };
        draw_circle(bullet.pos.x, bullet.pos.y, bullet.radius, color);
    }

    for t in &game.treasures {
        draw_circle(t.pos.x, t.pos.y, t.radius, t.color());
        let label = t.label();
        let dims = measure_text_ui(ui, label, 18);
        draw_text_ui(
            ui,
            label,
            t.pos.x - dims.width * 0.5,
            t.pos.y + dims.height * 0.35,
            18,
            BLACK,
        );
    }

    for p in &game.particles {
        let alpha = (p.life / 0.45).clamp(0.0, 1.0);
        let color = Color::new(p.color.r, p.color.g, p.color.b, alpha);
        draw_circle(p.pos.x, p.pos.y, p.radius, color);
    }
}

fn draw_paused(ui: &Ui, profile: &PlayerProfile, game: &Game) {
    draw_playing(ui, profile, game);
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_centered_text(ui, "暂停", 300.0, 48, WHITE);
    draw_centered_text(ui, "P/Enter: 继续", 360.0, 24, GRAY);
    draw_centered_text(ui, "M: 返回菜单", 400.0, 24, GRAY);
}

fn draw_menu(ui: &Ui, profile: &PlayerProfile) {
    clear_background(Color::new(0.04, 0.04, 0.08, 1.0));
    draw_centered_text(ui, "飞机大战", 160.0, 56, WHITE);
    draw_centered_text(ui, &format!("玩家: {}", profile.username), 240.0, 28, GRAY);
    draw_centered_text(
        ui,
        &format!(
            "永久升级: 子弹Lv{} / 最大生命{}",
            profile.permanent.bullet_level, profile.permanent.max_lives
        ),
        276.0,
        22,
        GRAY,
    );
    draw_centered_text(ui, "Enter: 开始游戏", 360.0, 28, WHITE);
    draw_centered_text(ui, "L: 排行榜", 400.0, 28, WHITE);
    draw_centered_text(ui, "1-5: 选择弹药  0: 取消", 440.0, 22, GRAY);
    draw_centered_text(ui, "I: 无敌  F: 自动发射", 470.0, 22, GRAY);
    draw_centered_text(ui, "Esc: 退出", 510.0, 28, WHITE);

    draw_centered_text(ui, "飞机等级随分数提升", 540.0, 22, GRAY);
    draw_centered_text(ui, "炼气期·初期", 568.0, 26, WHITE);

    let preview_pos = vec2(SCREEN_W * 0.5, 620.0);
    if let Some(sheet) = &ui.plane_sheet {
        let frame = plane_anim_frame();
        draw_plane_sprite(sheet, 0, frame, preview_pos, vec2(58.0, 72.0), false);
    } else {
        draw_plane(preview_pos, vec2(50.0, 62.0), LIME, false);
    }
}

fn draw_plane(pos: Vec2, size: Vec2, body: Color, boosted: bool) {
    let w = size.x;
    let h = size.y;
    let half_w = w * 0.5;
    let half_h = h * 0.5;

    let nose = vec2(pos.x, pos.y - half_h);
    let left_tail = vec2(pos.x - half_w * 0.65, pos.y + half_h * 0.75);
    let right_tail = vec2(pos.x + half_w * 0.65, pos.y + half_h * 0.75);

    let wing_span = w * 1.25;
    let wing_y = pos.y + h * 0.05;
    let wing_tip_left = vec2(pos.x - wing_span * 0.5, wing_y);
    let wing_tip_right = vec2(pos.x + wing_span * 0.5, wing_y);
    let wing_root_left = vec2(pos.x - w * 0.22, pos.y + h * 0.18);
    let wing_root_right = vec2(pos.x + w * 0.22, pos.y + h * 0.18);

    let tail_y = pos.y + h * 0.35;
    let tail_left = vec2(pos.x - w * 0.45, tail_y + h * 0.22);
    let tail_right = vec2(pos.x + w * 0.45, tail_y + h * 0.22);

    let cockpit = Color::new(
        (body.r + 0.25).min(1.0),
        (body.g + 0.25).min(1.0),
        (body.b + 0.4).min(1.0),
        1.0,
    );
    let accent = Color::new(
        (body.r + 0.15).min(1.0),
        (body.g + 0.1).min(1.0),
        (body.b + 0.1).min(1.0),
        1.0,
    );

    draw_triangle(nose, left_tail, right_tail, body);
    draw_triangle(wing_tip_left, wing_root_left, wing_root_right, accent);
    draw_triangle(wing_tip_right, wing_root_right, wing_root_left, accent);
    draw_triangle(vec2(pos.x, tail_y), tail_left, tail_right, accent);

    let cockpit_w = w * 0.28;
    let cockpit_h = h * 0.32;
    draw_ellipse(
        pos.x,
        pos.y - h * 0.12,
        cockpit_w * 0.5,
        cockpit_h * 0.5,
        0.0,
        cockpit,
    );

    let engine_y = pos.y + half_h * 0.55;
    draw_circle(pos.x - w * 0.15, engine_y, w * 0.08, DARKGRAY);
    draw_circle(pos.x + w * 0.15, engine_y, w * 0.08, DARKGRAY);
    draw_circle(pos.x - w * 0.15, engine_y, w * 0.045, ORANGE);
    draw_circle(pos.x + w * 0.15, engine_y, w * 0.045, ORANGE);

    if boosted {
        let glow = Color::new(0.4, 0.8, 1.0, 0.5);
        draw_circle(pos.x - w * 0.15, engine_y + h * 0.12, w * 0.12, glow);
        draw_circle(pos.x + w * 0.15, engine_y + h * 0.12, w * 0.12, glow);
    }
}

fn draw_plane_sprite(
    sheet: &PlaneSheet,
    level: usize,
    frame: usize,
    pos: Vec2,
    size: Vec2,
    boosted: bool,
) {
    let src = sheet.frame(level, frame);
    draw_texture_ex(
        &sheet.texture,
        pos.x - size.x * 0.5,
        pos.y - size.y * 0.5,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(src),
            ..Default::default()
        },
    );

    if boosted {
        let glow = Color::new(0.4, 0.8, 1.0, 0.45);
        draw_circle(pos.x - size.x * 0.15, pos.y + size.y * 0.3, size.x * 0.14, glow);
        draw_circle(pos.x + size.x * 0.15, pos.y + size.y * 0.3, size.x * 0.14, glow);
    }
}

fn draw_game_over(ui: &Ui, profile: &PlayerProfile, game: &Game) {
    clear_background(Color::new(0.08, 0.02, 0.02, 1.0));
    draw_centered_text(ui, "Game Over", 170.0, 56, WHITE);
    draw_centered_text(ui, &format!("玩家: {}", profile.username), 250.0, 28, GRAY);
    draw_centered_text(ui, &format!("本局得分: {}", game.score), 290.0, 32, YELLOW);
    draw_centered_text(ui, "Enter: 再来一局", 380.0, 28, WHITE);
    draw_centered_text(ui, "M: 返回菜单", 420.0, 28, WHITE);
    draw_centered_text(ui, "Esc: 退出", 460.0, 28, WHITE);
}

fn draw_leaderboard(ui: &Ui, lb: &Leaderboard) {
    clear_background(Color::new(0.03, 0.06, 0.04, 1.0));
    draw_centered_text(ui, "排行榜 (Top 10)", 110.0, 44, WHITE);
    let start_y = 170.0;
    for (i, e) in lb.entries.iter().take(10).enumerate() {
        let line = format!("{:>2}. {:<12}  {}", i + 1, e.username, e.score);
        draw_text_ui(ui, &line, 90.0, start_y + i as f32 * 32.0, 28, WHITE);
    }
    draw_centered_text(ui, "Esc: 返回菜单", 650.0, 24, GRAY);
}

fn draw_name_input(ui: &Ui, input: &NameInput) {
    clear_background(Color::new(0.02, 0.02, 0.02, 1.0));
    draw_centered_text(ui, "首次运行：输入用户名", 220.0, 36, WHITE);
    draw_centered_text(ui, "Enter 确认 / Backspace 删除", 270.0, 22, GRAY);

    let box_w = 320.0;
    let box_h = 46.0;
    let box_x = (SCREEN_W - box_w) * 0.5;
    let box_y = 320.0;
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, WHITE);

    let shown = if input.buffer.is_empty() {
        "..."
    } else {
        input.buffer.as_str()
    };
    draw_text_ui(ui, shown, box_x + 14.0, box_y + 32.0, 30, WHITE);

    if let Some((msg, until)) = &input.error {
        if get_time() <= *until {
            draw_centered_text(ui, msg, 400.0, 22, PINK);
        }
    }
}

fn update_name_input(input: &mut NameInput) -> Option<String> {
    while let Some(ch) = get_char_pressed() {
        if ch.is_control() {
            continue;
        }
        if input.buffer.chars().count() >= 12 {
            continue;
        }
        input.buffer.push(ch);
    }

    if is_key_pressed(KeyCode::Backspace) {
        input.buffer.pop();
    }

    if is_key_pressed(KeyCode::Enter) {
        let trimmed = input.buffer.trim().to_string();
        if trimmed.is_empty() {
            input.error = Some(("用户名不能为空".to_string(), get_time() + 1.2));
            return None;
        }
        return Some(trimmed);
    }

    None
}

async fn load_ui_font() -> Option<Font> {
    let candidates = [
        "assets/NotoSansSC-Regular.ttf",
        "assets/NotoSansSC-Regular.otf",
        "assets/msyh.ttf",
        "desktop/assets/NotoSansSC-Regular.ttf",
        "desktop/assets/NotoSansSC-Regular.otf",
        "desktop/assets/msyh.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode MS.ttf",
        "/Library/Fonts/Microsoft YaHei.ttf",
        "C:/Windows/Fonts/simhei.ttf",
        "C:/Windows/Fonts/msyh.ttf",
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simsun.ttc",
    ];

    for path in candidates {
        if !Path::new(path).exists() {
            continue;
        }
        if let Ok(font) = load_ttf_font(path).await {
            return Some(font);
        }
    }

    None
}

async fn load_plane_sheet() -> Option<PlaneSheet> {
    let primary = "assets/planes_levels.png";
    let fallback = "assets/planes.png";
    let path = if Path::new(primary).exists() {
        primary
    } else if Path::new(fallback).exists() {
        fallback
    } else {
        return None;
    };

    let texture = load_texture(path).await.ok()?;
    texture.set_filter(FilterMode::Nearest);

    let cols = PLANE_FRAMES;
    let rows = if path == primary { PLANE_LEVELS } else { 5 };
    let frame_w = texture.width() / cols as f32;
    let frame_h = texture.height() / rows as f32;

    Some(PlaneSheet {
        texture,
        cols,
        rows,
        frame_w,
        frame_h,
    })
}

#[macroquad::main(window_conf)]
async fn main() {
    let store = SaveStore::new();

    let mut profile = store.load_profile().ok().flatten().unwrap_or_default();
    let mut leaderboard = store.load_leaderboard().unwrap_or_default();
    let ui = Ui {
        font: load_ui_font().await,
        plane_sheet: load_plane_sheet().await,
    };

    let mut mode = if profile.username.trim().is_empty() {
        AppMode::EnterName
    } else {
        AppMode::Menu
    };

    let mut name_input = NameInput::default();
    let mut game = Game::new(&profile);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            match mode {
                AppMode::EnterName => {}
                AppMode::Menu => break,
                AppMode::Leaderboard => mode = AppMode::Menu,
                AppMode::Playing => mode = AppMode::Menu,
                AppMode::Paused => mode = AppMode::Menu,
                AppMode::GameOver => mode = AppMode::Menu,
            }
        }

        match mode {
            AppMode::EnterName => {
                if let Some(name) = update_name_input(&mut name_input) {
                    profile.username = name;
                    if let Err(err) = store.save_profile(&profile) {
                        name_input.error = Some((format!("保存失败: {err}"), get_time() + 2.0));
                    } else {
                        game = Game::new(&profile);
                        mode = AppMode::Menu;
                    }
                }
                draw_name_input(&ui, &name_input);
            }
            AppMode::Menu => {
                if is_key_pressed(KeyCode::Enter) {
                    game = Game::new(&profile);
                    mode = AppMode::Playing;
                } else if is_key_pressed(KeyCode::L) {
                    mode = AppMode::Leaderboard;
                }
                draw_menu(&ui, &profile);
            }
            AppMode::Leaderboard => {
                draw_leaderboard(&ui, &leaderboard);
            }
            AppMode::Playing => {
                if is_key_pressed(KeyCode::P) {
                    mode = AppMode::Paused;
                    draw_paused(&ui, &profile, &game);
                    next_frame().await;
                    continue;
                }
                let game_over = match update_playing(&store, &mut profile, &mut leaderboard, &mut game) {
                    Ok(v) => v,
                    Err(_) => true,
                };
                draw_playing(&ui, &profile, &game);
                if game_over {
                    mode = AppMode::GameOver;
                    game.game_over_cooldown = 0.3;
                }
            }
            AppMode::Paused => {
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Enter) {
                    mode = AppMode::Playing;
                } else if is_key_pressed(KeyCode::M) {
                    mode = AppMode::Menu;
                }
                draw_paused(&ui, &profile, &game);
            }
            AppMode::GameOver => {
                game.game_over_cooldown = (game.game_over_cooldown - get_frame_time()).max(0.0);
                if game.game_over_cooldown <= 0.0 {
                    if is_key_pressed(KeyCode::Enter) {
                        game = Game::new(&profile);
                        mode = AppMode::Playing;
                    } else if is_key_pressed(KeyCode::M) {
                        mode = AppMode::Menu;
                    }
                }
                draw_game_over(&ui, &profile, &game);
            }
        }

        next_frame().await;
    }
}
