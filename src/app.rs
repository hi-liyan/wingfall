use macroquad::prelude::*;

use crate::actors::player::Player;
use crate::assets::load_ui_font;
use crate::config::{INTERNAL_RENDER_SCALE, SCREEN_H, SCREEN_W};
use crate::render::{draw_hud, draw_map, draw_player, draw_portals};
use crate::systems::{handle_interaction, handle_movement};
use crate::ui::Ui;
use crate::world::World;

// 游戏主循环：加载地图数据，处理输入与渲染
pub async fn run() {
    let ui = Ui {
        font: load_ui_font().await,
    };

    // 加载地图配置（数据驱动）
    let mut world = World::load_from_file("data/maps.json").unwrap_or_else(|_| World::default());

    // 初始化玩家位置到当前地图的出生点
    let mut player = Player::new(world.current_spawn());

    // 低分辨率渲染目标，用于像素风文字
    let rt_w = (SCREEN_W * INTERNAL_RENDER_SCALE).max(1.0) as u32;
    let rt_h = (SCREEN_H * INTERNAL_RENDER_SCALE).max(1.0) as u32;
    let rt_w_f = rt_w as f32;
    let rt_h_f = rt_h as f32;
    let render_target = render_target(rt_w, rt_h);
    render_target.texture.set_filter(FilterMode::Nearest);

    loop {
        // 移动与交互
        handle_movement(&mut player);
        handle_interaction(&mut world, &mut player);

        // 计算窗口缩放
        let (scale, offset_x, offset_y) = compute_viewport();
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W, SCREEN_H));
        camera.render_target = Some(render_target.clone());
        set_camera(&camera);

        // 绘制当前地图与实体
        draw_map(&ui, world.current_map());
        draw_portals(&ui, world.current_map());
        draw_player(&ui, &player);
        draw_hud(&ui, world.current_map());

        // 回到默认相机并放大显示
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &render_target.texture,
            offset_x,
            offset_y,
            WHITE,
            DrawTextureParams {
                // RenderTarget 在纹理坐标系中是倒置的，这里做一次垂直翻转
                source: Some(Rect::new(0.0, rt_h_f, rt_w_f, -rt_h_f)),
                dest_size: Some(vec2(SCREEN_W * scale, SCREEN_H * scale)),
                ..Default::default()
            },
        );

        next_frame().await;
    }
}

// 根据窗口尺寸计算缩放比例与居中偏移
fn compute_viewport() -> (f32, f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    let scale = (sw / SCREEN_W).min(sh / SCREEN_H).max(0.1);
    let offset_x = (sw - SCREEN_W * scale) * 0.5;
    let offset_y = (sh - SCREEN_H * scale) * 0.5;
    (scale, offset_x, offset_y)
}
