use std::path::Path;

use macroquad::prelude::*;

// 加载UI字体，优先使用项目资源中的字体，其次尝试系统字体
pub async fn load_ui_font() -> Option<Font> {
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
        // 跳过不存在的候选路径
        if !Path::new(path).exists() {
            continue;
        }
        // 成功加载即可返回
        if let Ok(font) = load_ttf_font(path).await {
            return Some(font);
        }
    }

    None
}
