use image::{ImageBuffer, Rgba, RgbaImage};
use std::f32::consts::TAU;

const FRAME_SIZE: u32 = 128;
const FRAMES: usize = 8;
const LEVELS: usize = 15; // 5 realms * 3 phases

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy)]
struct Palette {
    body: Color,
    energy: Color,
    glow: Color,
    plasma: Color,
    lightning: Color,
}

#[derive(Clone, Copy)]
struct LevelSpec {
    name: &'static str,
    palette: Palette,
    intensity: f32,
    aura_layers: u8,
    lightning: u8,
    afterimage: u8,
}

fn blend(dst: Color, src: Color) -> Color {
    if src.a == 255 {
        return src;
    }
    let sa = src.a as f32 / 255.0;
    let da = dst.a as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return Color::rgba(0, 0, 0, 0);
    }
    let r = (src.r as f32 * sa + dst.r as f32 * da * (1.0 - sa)) / out_a;
    let g = (src.g as f32 * sa + dst.g as f32 * da * (1.0 - sa)) / out_a;
    let b = (src.b as f32 * sa + dst.b as f32 * da * (1.0 - sa)) / out_a;
    Color::rgba(r as u8, g as u8, b as u8, (out_a * 255.0) as u8)
}

fn set_px(img: &mut RgbaImage, x: i32, y: i32, color: Color) {
    if x < 0 || y < 0 {
        return;
    }
    let (w, h) = img.dimensions();
    if x as u32 >= w || y as u32 >= h {
        return;
    }
    let dst = img.get_pixel(x as u32, y as u32);
    let dst = Color::rgba(dst[0], dst[1], dst[2], dst[3]);
    let out = blend(dst, color);
    img.put_pixel(x as u32, y as u32, Rgba([out.r, out.g, out.b, out.a]));
}

fn fill_rect(img: &mut RgbaImage, cx: i32, cy: i32, hw: i32, hh: i32, color: Color) {
    for y in (cy - hh)..=(cy + hh) {
        for x in (cx - hw)..=(cx + hw) {
            set_px(img, x, y, color);
        }
    }
}

fn fill_ellipse(img: &mut RgbaImage, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
    let rx2 = (rx * rx) as f32;
    let ry2 = (ry * ry) as f32;
    for y in (cy - ry)..=(cy + ry) {
        for x in (cx - rx)..=(cx + rx) {
            let dx = (x - cx) as f32;
            let dy = (y - cy) as f32;
            if (dx * dx) / rx2 + (dy * dy) / ry2 <= 1.0 {
                set_px(img, x, y, color);
            }
        }
    }
}

fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Color) {
    fill_ellipse(img, cx, cy, r, r, color);
}

fn fill_triangle(img: &mut RgbaImage, p0: (i32, i32), p1: (i32, i32), p2: (i32, i32), color: Color) {
    let min_x = p0.0.min(p1.0).min(p2.0);
    let max_x = p0.0.max(p1.0).max(p2.0);
    let min_y = p0.1.min(p1.1).min(p2.1);
    let max_y = p0.1.max(p1.1).max(p2.1);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let b0 = edge(p1, p2, (x, y));
            let b1 = edge(p2, p0, (x, y));
            let b2 = edge(p0, p1, (x, y));
            if (b0 >= 0 && b1 >= 0 && b2 >= 0) || (b0 <= 0 && b1 <= 0 && b2 <= 0) {
                set_px(img, x, y, color);
            }
        }
    }
}

fn edge(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i32 {
    (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
}

fn draw_line(img: &mut RgbaImage, a: (i32, i32), b: (i32, i32), color: Color) {
    let mut x0 = a.0;
    let mut y0 = a.1;
    let x1 = b.0;
    let y1 = b.1;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        set_px(img, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn rand_u32(seed: u32) -> u32 {
    let mut x = seed ^ 0x9e37_79b9;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

fn rand_f32(seed: u32) -> f32 {
    (rand_u32(seed) as f32) / (u32::MAX as f32)
}

fn levels() -> [LevelSpec; LEVELS] {
    use Color as C;
    [
        // 炼气期
        LevelSpec {
            name: "炼气-初",
            palette: Palette {
                body: C::rgba(52, 58, 66, 255),
                energy: C::rgba(120, 190, 255, 220),
                glow: C::rgba(90, 180, 255, 180),
                plasma: C::rgba(120, 130, 255, 210),
                lightning: C::rgba(80, 160, 255, 200),
            },
            intensity: 0.6,
            aura_layers: 0,
            lightning: 0,
            afterimage: 0,
        },
        LevelSpec {
            name: "炼气-中",
            palette: Palette {
                body: C::rgba(54, 60, 70, 255),
                energy: C::rgba(100, 200, 255, 230),
                glow: C::rgba(120, 200, 255, 200),
                plasma: C::rgba(130, 140, 255, 220),
                lightning: C::rgba(100, 200, 255, 200),
            },
            intensity: 0.8,
            aura_layers: 1,
            lightning: 1,
            afterimage: 0,
        },
        LevelSpec {
            name: "炼气-后",
            palette: Palette {
                body: C::rgba(50, 58, 68, 240),
                energy: C::rgba(130, 210, 255, 230),
                glow: C::rgba(140, 220, 255, 210),
                plasma: C::rgba(150, 120, 255, 230),
                lightning: C::rgba(160, 200, 255, 220),
            },
            intensity: 1.0,
            aura_layers: 1,
            lightning: 2,
            afterimage: 1,
        },
        // 筑基期
        LevelSpec {
            name: "筑基-初",
            palette: Palette {
                body: C::rgba(56, 60, 72, 255),
                energy: C::rgba(150, 220, 255, 230),
                glow: C::rgba(160, 230, 255, 210),
                plasma: C::rgba(140, 150, 255, 220),
                lightning: C::rgba(150, 220, 255, 210),
            },
            intensity: 1.1,
            aura_layers: 2,
            lightning: 1,
            afterimage: 1,
        },
        LevelSpec {
            name: "筑基-中",
            palette: Palette {
                body: C::rgba(54, 58, 70, 255),
                energy: C::rgba(120, 220, 255, 230),
                glow: C::rgba(150, 235, 255, 220),
                plasma: C::rgba(120, 170, 255, 230),
                lightning: C::rgba(120, 220, 255, 220),
            },
            intensity: 1.25,
            aura_layers: 2,
            lightning: 2,
            afterimage: 1,
        },
        LevelSpec {
            name: "筑基-后",
            palette: Palette {
                body: C::rgba(52, 56, 68, 255),
                energy: C::rgba(160, 230, 255, 240),
                glow: C::rgba(180, 240, 255, 230),
                plasma: C::rgba(130, 200, 255, 230),
                lightning: C::rgba(200, 220, 255, 230),
            },
            intensity: 1.5,
            aura_layers: 3,
            lightning: 3,
            afterimage: 1,
        },
        // 结丹期
        LevelSpec {
            name: "结丹-初",
            palette: Palette {
                body: C::rgba(56, 60, 72, 255),
                energy: C::rgba(170, 200, 255, 230),
                glow: C::rgba(190, 210, 255, 220),
                plasma: C::rgba(230, 200, 120, 220),
                lightning: C::rgba(200, 200, 255, 230),
            },
            intensity: 1.6,
            aura_layers: 2,
            lightning: 2,
            afterimage: 1,
        },
        LevelSpec {
            name: "结丹-中",
            palette: Palette {
                body: C::rgba(58, 62, 74, 255),
                energy: C::rgba(190, 210, 255, 235),
                glow: C::rgba(210, 220, 255, 230),
                plasma: C::rgba(240, 190, 120, 230),
                lightning: C::rgba(200, 210, 255, 230),
            },
            intensity: 1.8,
            aura_layers: 3,
            lightning: 3,
            afterimage: 1,
        },
        LevelSpec {
            name: "结丹-后",
            palette: Palette {
                body: C::rgba(60, 64, 76, 255),
                energy: C::rgba(200, 220, 255, 240),
                glow: C::rgba(220, 230, 255, 240),
                plasma: C::rgba(250, 200, 120, 235),
                lightning: C::rgba(220, 220, 255, 235),
            },
            intensity: 2.0,
            aura_layers: 3,
            lightning: 4,
            afterimage: 2,
        },
        // 元婴期
        LevelSpec {
            name: "元婴-初",
            palette: Palette {
                body: C::rgba(54, 58, 70, 250),
                energy: C::rgba(170, 160, 255, 235),
                glow: C::rgba(190, 180, 255, 230),
                plasma: C::rgba(140, 120, 255, 230),
                lightning: C::rgba(170, 160, 255, 230),
            },
            intensity: 2.1,
            aura_layers: 3,
            lightning: 3,
            afterimage: 2,
        },
        LevelSpec {
            name: "元婴-中",
            palette: Palette {
                body: C::rgba(52, 56, 68, 245),
                energy: C::rgba(190, 170, 255, 240),
                glow: C::rgba(210, 190, 255, 235),
                plasma: C::rgba(160, 120, 255, 235),
                lightning: C::rgba(190, 170, 255, 235),
            },
            intensity: 2.4,
            aura_layers: 4,
            lightning: 4,
            afterimage: 2,
        },
        LevelSpec {
            name: "元婴-后",
            palette: Palette {
                body: C::rgba(50, 54, 66, 240),
                energy: C::rgba(210, 180, 255, 245),
                glow: C::rgba(230, 200, 255, 240),
                plasma: C::rgba(180, 120, 255, 240),
                lightning: C::rgba(220, 200, 255, 240),
            },
            intensity: 2.8,
            aura_layers: 4,
            lightning: 5,
            afterimage: 3,
        },
        // 化神期
        LevelSpec {
            name: "化神-初",
            palette: Palette {
                body: C::rgba(56, 60, 72, 250),
                energy: C::rgba(220, 210, 255, 245),
                glow: C::rgba(230, 230, 255, 240),
                plasma: C::rgba(200, 200, 255, 240),
                lightning: C::rgba(230, 230, 255, 240),
            },
            intensity: 3.0,
            aura_layers: 4,
            lightning: 5,
            afterimage: 3,
        },
        LevelSpec {
            name: "化神-中",
            palette: Palette {
                body: C::rgba(54, 58, 70, 240),
                energy: C::rgba(235, 225, 255, 245),
                glow: C::rgba(245, 245, 255, 240),
                plasma: C::rgba(210, 210, 255, 245),
                lightning: C::rgba(240, 240, 255, 245),
            },
            intensity: 3.4,
            aura_layers: 5,
            lightning: 6,
            afterimage: 3,
        },
        LevelSpec {
            name: "化神-后",
            palette: Palette {
                body: C::rgba(52, 56, 68, 235),
                energy: C::rgba(245, 235, 255, 250),
                glow: C::rgba(255, 255, 255, 245),
                plasma: C::rgba(230, 230, 255, 245),
                lightning: C::rgba(255, 255, 255, 245),
            },
            intensity: 4.0,
            aura_layers: 6,
            lightning: 7,
            afterimage: 4,
        },
    ]
}

fn draw_plane_base(img: &mut RgbaImage, cx: i32, cy: i32, body: Color) {
    fill_ellipse(img, cx, cy - 4, 12, 40, body);
    fill_triangle(img, (cx, cy - 50), (cx - 7, cy - 30), (cx + 7, cy - 30), body);
    fill_rect(img, cx, cy + 28, 5, 10, body);
    fill_rect(img, cx, cy + 36, 18, 4, body);
    fill_rect(img, cx, cy - 4, 36, 6, body);
    fill_triangle(img, (cx - 36, cy - 4), (cx - 22, cy - 14), (cx - 22, cy + 4), body);
    fill_triangle(img, (cx + 36, cy - 4), (cx + 22, cy - 14), (cx + 22, cy + 4), body);
    fill_rect(img, cx, cy + 10, 18, 3, body);
    fill_triangle(img, (cx - 18, cy + 10), (cx - 6, cy + 18), (cx - 6, cy + 6), body);
    fill_triangle(img, (cx + 18, cy + 10), (cx + 6, cy + 18), (cx + 6, cy + 6), body);
}

fn draw_cockpit(img: &mut RgbaImage, cx: i32, cy: i32) {
    fill_ellipse(img, cx, cy - 18, 5, 10, Color::rgba(80, 180, 220, 220));
    fill_ellipse(img, cx, cy - 18, 2, 5, Color::rgba(15, 20, 25, 200));
}

fn draw_engines(img: &mut RgbaImage, cx: i32, cy: i32) {
    fill_circle(img, cx - 8, cy + 22, 6, Color::rgba(40, 42, 46, 255));
    fill_circle(img, cx + 8, cy + 22, 6, Color::rgba(40, 42, 46, 255));
    fill_circle(img, cx - 8, cy + 22, 3, Color::rgba(255, 140, 60, 220));
    fill_circle(img, cx + 8, cy + 22, 3, Color::rgba(255, 140, 60, 220));
}

fn draw_details(img: &mut RgbaImage, cx: i32, cy: i32, energy: Color, glow: Color) {
    draw_line(img, (cx - 26, cy - 2), (cx - 12, cy - 2), glow);
    draw_line(img, (cx + 12, cy - 2), (cx + 26, cy - 2), glow);
    draw_line(img, (cx - 20, cy + 6), (cx - 8, cy + 6), energy);
    draw_line(img, (cx + 8, cy + 6), (cx + 20, cy + 6), energy);
    draw_line(img, (cx, cy - 40), (cx, cy + 14), Color::rgba(90, 92, 100, 180));
    for i in -2..=2 {
        set_px(img, cx + i * 2, cy + 2, Color::rgba(110, 110, 120, 200));
    }
    set_px(img, cx - 6, cy - 8, energy);
    set_px(img, cx - 8, cy - 6, energy);
    set_px(img, cx + 6, cy - 8, energy);
    set_px(img, cx + 8, cy - 6, energy);
    set_px(img, cx - 16, cy + 12, Color::rgba(200, 200, 200, 200));
    set_px(img, cx - 14, cy + 12, Color::rgba(200, 200, 200, 200));
    set_px(img, cx - 16, cy + 14, Color::rgba(200, 200, 200, 200));
    set_px(img, cx + 14, cy + 12, energy);
    set_px(img, cx + 16, cy + 12, energy);
    set_px(img, cx + 16, cy + 14, energy);
}

fn draw_aura(img: &mut RgbaImage, cx: i32, cy: i32, phase: f32, spec: &LevelSpec) {
    for i in 0..spec.aura_layers {
        let r = 28 + i as i32 * 6 + (phase.cos() * 2.0) as i32;
        let alpha = (120.0 / (i as f32 + 1.0)).min(180.0) as u8;
        let mut glow = spec.palette.glow;
        glow.a = alpha;
        fill_ellipse(img, cx, cy - 6, r, r + 8, glow);
    }
}

fn draw_afterimages(img: &mut RgbaImage, cx: i32, cy: i32, phase: f32, spec: &LevelSpec) {
    for i in 0..spec.afterimage {
        let shift = ((phase + i as f32) * 2.0).sin() * 6.0;
        let alpha = (90.0 / (i as f32 + 1.0)) as u8;
        let mut ghost = spec.palette.glow;
        ghost.a = alpha;
        fill_ellipse(img, cx + shift as i32, cy - 4, 10, 32, ghost);
    }
}

fn draw_particles(img: &mut RgbaImage, cx: i32, cy: i32, frame: usize, spec: &LevelSpec) {
    let count = (8.0 * spec.intensity) as i32;
    for i in 0..count {
        let seed = (frame as u32) * 31 + i as u32 * 131 + (spec.intensity as u32) * 17;
        let ang = rand_f32(seed) * TAU;
        let radius = 12.0 + rand_f32(seed ^ 0x55aa) * 28.0;
        let px = cx as f32 + ang.cos() * radius;
        let py = cy as f32 + ang.sin() * radius * 0.7;
        let size = 1 + (rand_f32(seed ^ 0xa5a5) * 2.0) as i32;
        let mut color = spec.palette.energy;
        color.a = (120.0 + rand_f32(seed ^ 0x1234) * 90.0) as u8;
        fill_circle(img, px as i32, py as i32, size, color);
    }
}

fn draw_lightning(img: &mut RgbaImage, cx: i32, cy: i32, frame: usize, spec: &LevelSpec) {
    for i in 0..spec.lightning {
        let seed = (frame as u32) * 97 + i as u32 * 151 + (spec.intensity as u32) * 29;
        let x0 = cx - 26 + (rand_f32(seed) * 52.0) as i32;
        let y0 = cy - 6 + (rand_f32(seed ^ 0x51) * 20.0) as i32;
        let x1 = x0 + (rand_f32(seed ^ 0x99) * 18.0 - 9.0) as i32;
        let y1 = y0 + (rand_f32(seed ^ 0x77) * 22.0 + 8.0) as i32;
        draw_line(img, (x0, y0), (x1, y1), spec.palette.lightning);
        draw_line(img, (x0 + 1, y0), (x1 + 1, y1), spec.palette.glow);
    }
}

fn draw_flames(img: &mut RgbaImage, cx: i32, cy: i32, phase: f32, spec: &LevelSpec) {
    let flame_len = 10 + (phase.sin() * 6.0).round() as i32;
    let base_y = cy + 28;
    let mut outer = spec.palette.plasma;
    outer.a = 210;
    let mut inner = spec.palette.lightning;
    inner.a = 210;
    fill_triangle(img, (cx - 8, base_y), (cx - 13, base_y + flame_len), (cx - 3, base_y + flame_len), outer);
    fill_triangle(img, (cx + 8, base_y), (cx + 3, base_y + flame_len), (cx + 13, base_y + flame_len), outer);
    fill_triangle(img, (cx - 8, base_y), (cx - 10, base_y + flame_len / 2), (cx - 6, base_y + flame_len / 2), inner);
    fill_triangle(img, (cx + 8, base_y), (cx + 6, base_y + flame_len / 2), (cx + 10, base_y + flame_len / 2), inner);
}

fn draw_laser(img: &mut RgbaImage, cx: i32, cy: i32, frame: usize, spec: &LevelSpec) {
    let wobble = (frame as i32 % 2) * 2;
    let width = (2.0 + spec.intensity).min(6.0) as i32;
    let length = 30 + (spec.intensity * 6.0) as i32;
    let color = spec.palette.energy;
    for i in 0..width {
        let offset = i - width / 2;
        draw_line(
            img,
            (cx + offset + wobble, cy - 40),
            (cx + offset + wobble, cy - 40 - length),
            color,
        );
    }
}

fn draw_plane_frame(img: &mut RgbaImage, level: &LevelSpec, frame: usize, ox: i32, oy: i32) {
    let phase = frame as f32 / FRAMES as f32 * TAU;
    let bob = (phase.sin() * 1.5).round() as i32;
    let cx = ox + 64;
    let cy = oy + 64 + bob;

    draw_aura(img, cx, cy, phase, level);
    draw_afterimages(img, cx, cy, phase, level);

    draw_plane_base(img, cx, cy, level.palette.body);
    draw_cockpit(img, cx, cy);
    draw_engines(img, cx, cy);
    draw_details(img, cx, cy, level.palette.energy, level.palette.glow);

    draw_particles(img, cx, cy, frame, level);
    draw_lightning(img, cx, cy, frame, level);
    draw_flames(img, cx, cy, phase, level);
    draw_laser(img, cx, cy, frame, level);
}

fn main() -> Result<(), String> {
    let width = FRAME_SIZE as u32 * FRAMES as u32;
    let height = FRAME_SIZE as u32 * LEVELS as u32;
    let mut img: RgbaImage = ImageBuffer::new(width, height);

    let levels = levels();
    for (row, level) in levels.iter().enumerate() {
        for frame in 0..FRAMES {
            let ox = (frame as i32) * FRAME_SIZE as i32;
            let oy = (row as i32) * FRAME_SIZE as i32;
            draw_plane_frame(&mut img, level, frame, ox, oy);
        }
    }

    img.save("assets/planes_levels.png")
        .map_err(|e| format!("save failed: {e}"))?;
    println!(
        "wrote assets/planes_levels.png ({}x{}, {} rows)",
        width, height, LEVELS
    );
    Ok(())
}
