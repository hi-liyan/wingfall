use macroquad::prelude::*;

#[derive(Default)]
pub struct Ui {
    pub font: Option<Font>,
}

impl Ui {
    // 获取字体引用，便于统一绘制接口
    pub fn font(&self) -> Option<&Font> {
        self.font.as_ref()
    }
}

// 绘制UI文字，优先使用加载的字体
pub fn draw_text_ui(ui: &Ui, text: &str, x: f32, y: f32, size: u16, color: Color) {
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

// 测量文字宽高，用于布局计算
pub fn measure_text_ui(ui: &Ui, text: &str, size: u16) -> TextDimensions {
    measure_text(text, ui.font(), size, 1.0)
}

// 绘制水平居中的文字
pub fn draw_centered_text(ui: &Ui, text: &str, y: f32, size: u16, color: Color) {
    let dims = measure_text_ui(ui, text, size);
    draw_text_ui(ui, text, (crate::config::SCREEN_W - dims.width) * 0.5, y, size, color);
}
