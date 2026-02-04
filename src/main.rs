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
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppMode {
    EnterName,
    Menu,
    Leaderboard,
    Playing,
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
            TreasureKind::MaxLifePermanent => "♥P",
            TreasureKind::LifePlus => "♥",
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
    score: u32,
    enemy_spawn_timer: f32,
    enemy_spawn_interval: f32,
    game_over_cooldown: f32,
    just_saved_score: bool,
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
            temp_mode: None,
        };
        Self {
            player,
            bullets: Vec::new(),
            enemies: Vec::new(),
            treasures: Vec::new(),
            score: 0,
            enemy_spawn_timer: 0.0,
            enemy_spawn_interval: 0.85,
            game_over_cooldown: 0.0,
            just_saved_score: false,
        }
    }
}

#[derive(Default)]
struct NameInput {
    buffer: String,
    error: Option<(String, f64)>,
}

struct Ui {
    font: Option<Font>,
}

impl Ui {
    fn font(&self) -> Option<&Font> {
        self.font.as_ref()
    }
}

fn clamp_vec2(v: Vec2, min: Vec2, max: Vec2) -> Vec2 {
    vec2(v.x.clamp(min.x, max.x), v.y.clamp(min.y, max.y))
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
    if (is_key_down(KeyCode::Space) || is_mouse_button_down(MouseButton::Left))
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

    let hud = format!(
        "{}  Score: {}  Lives: {}/{}",
        profile.username, game.score, game.player.lives, game.player.max_lives
    );
    draw_text_ui(ui, &hud, 12.0, 26.0, 24, WHITE);

    if game.player.is_invincible() {
        draw_text_ui(ui, "INVINCIBLE", 12.0, 50.0, 20, SKYBLUE);
    }
    if let Some((mode, until)) = game.player.temp_mode {
        let remain = (until - get_time()).max(0.0);
        let label = match mode {
            BulletMode::Spread => "SPREAD",
            BulletMode::Laser => "LASER",
            _ => "POWER",
        };
        draw_text_ui(
            ui,
            &format!("{label} {:.1}s", remain),
            12.0,
            72.0,
            20,
            GREEN,
        );
    }

    let pr = game.player.rect();
    let player_color = if game.player.is_invincible() { SKYBLUE } else { LIME };
    draw_triangle(
        vec2(pr.x + pr.w * 0.5, pr.y),
        vec2(pr.x, pr.y + pr.h),
        vec2(pr.x + pr.w, pr.y + pr.h),
        player_color,
    );

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
    draw_centered_text(ui, "Esc: 退出", 440.0, 28, WHITE);
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

#[macroquad::main(window_conf)]
async fn main() {
    let store = SaveStore::new();

    let mut profile = store.load_profile().ok().flatten().unwrap_or_default();
    let mut leaderboard = store.load_leaderboard().unwrap_or_default();
    let ui = Ui {
        font: load_ui_font().await,
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
