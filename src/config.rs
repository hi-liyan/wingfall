use macroquad::prelude::Conf;

pub const SCREEN_W: f32 = 960.0;
pub const SCREEN_H: f32 = 540.0;

// 降低内部渲染分辨率，放大时呈现像素风文本与画面
pub const INTERNAL_RENDER_SCALE: f32 = 0.5;

// 配置窗口标题、尺寸与可变大小选项
pub fn window_conf() -> Conf {
    let resizable = cfg!(any(target_os = "windows", target_os = "macos", target_os = "linux"));
    Conf {
        window_title: "凡人修仙传 像素版".to_string(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        high_dpi: true,
        window_resizable: resizable,
        ..Default::default()
    }
}
