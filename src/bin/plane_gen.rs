use image::{ImageBuffer, Rgba, RgbaImage};
use std::f32::consts::TAU;

const FRAME_SIZE: u32 = 128;
const FRAMES: usize = 8;
const STYLES: usize = 5;

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
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

fn draw_plane_frame(img: &mut RgbaImage, style: usize, frame: usize, ox: i32, oy: i32) {
    let phase = frame as f32 / FRAMES as f32 * TAU;
    let bob = (phase.sin() * 1.5).round() as i32;
    let flame_len = 10 + ((phase.sin() * 0.5 + 0.5) * 6.0).round() as i32;

    let base = [
        Color::rgba(45, 48, 55, 255),
        Color::rgba(60, 64, 70, 255),
        Color::rgba(70, 72, 78, 255),
        Color::rgba(52, 56, 62, 255),
        Color::rgba(66, 68, 74, 255),
    ];
    let accent = [
        Color::rgba(20, 180, 255, 255),
        Color::rgba(255, 140, 30, 255),
        Color::rgba(70, 220, 255, 255),
        Color::rgba(255, 110, 60, 255),
        Color::rgba(120, 200, 255, 255),
    ];
    let glow = [
        Color::rgba(80, 220, 255, 200),
        Color::rgba(255, 180, 80, 200),
        Color::rgba(120, 240, 255, 200),
        Color::rgba(255, 150, 100, 200),
        Color::rgba(140, 220, 255, 200),
    ];
    let body = base[style % base.len()];
    let energy = accent[style % accent.len()];
    let edge = glow[style % glow.len()];

    let cx = ox + 64;
    let cy = oy + 64 + bob;

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

    fill_ellipse(img, cx, cy - 18, 5, 10, Color::rgba(80, 180, 220, 220));
    fill_ellipse(img, cx, cy - 18, 2, 5, Color::rgba(15, 20, 25, 200));

    fill_circle(img, cx - 8, cy + 22, 6, Color::rgba(40, 42, 46, 255));
    fill_circle(img, cx + 8, cy + 22, 6, Color::rgba(40, 42, 46, 255));
    fill_circle(img, cx - 8, cy + 22, 3, Color::rgba(255, 140, 60, 220));
    fill_circle(img, cx + 8, cy + 22, 3, Color::rgba(255, 140, 60, 220));

    let flame_color = Color::rgba(110, 160, 255, 200);
    let flame_core = Color::rgba(180, 90, 255, 200);
    fill_triangle(
        img,
        (cx - 8, cy + 28),
        (cx - 13, cy + 28 + flame_len),
        (cx - 3, cy + 28 + flame_len),
        flame_color,
    );
    fill_triangle(
        img,
        (cx + 8, cy + 28),
        (cx + 3, cy + 28 + flame_len),
        (cx + 13, cy + 28 + flame_len),
        flame_color,
    );
    fill_triangle(
        img,
        (cx - 8, cy + 28),
        (cx - 10, cy + 28 + flame_len / 2),
        (cx - 6, cy + 28 + flame_len / 2),
        flame_core,
    );
    fill_triangle(
        img,
        (cx + 8, cy + 28),
        (cx + 6, cy + 28 + flame_len / 2),
        (cx + 10, cy + 28 + flame_len / 2),
        flame_core,
    );

    fill_circle(img, cx - 30, cy + 8, 4, Color::rgba(30, 34, 40, 255));
    fill_circle(img, cx + 30, cy + 8, 4, Color::rgba(30, 34, 40, 255));
    fill_rect(img, cx - 30, cy + 14, 4, 2, Color::rgba(25, 28, 34, 255));
    fill_rect(img, cx + 30, cy + 14, 4, 2, Color::rgba(25, 28, 34, 255));

    draw_line(img, (cx - 26, cy - 2), (cx - 12, cy - 2), edge);
    draw_line(img, (cx + 12, cy - 2), (cx + 26, cy - 2), edge);
    draw_line(img, (cx - 20, cy + 6), (cx - 8, cy + 6), energy);
    draw_line(img, (cx + 8, cy + 6), (cx + 20, cy + 6), energy);
    draw_line(img, (cx, cy - 40), (cx, cy + 14), Color::rgba(80, 82, 90, 180));

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

fn main() -> Result<(), String> {
    let width = FRAME_SIZE as u32 * FRAMES as u32;
    let height = FRAME_SIZE as u32 * STYLES as u32;
    let mut img: RgbaImage = ImageBuffer::new(width, height);

    for style in 0..STYLES {
        for frame in 0..FRAMES {
            let ox = (frame as i32) * FRAME_SIZE as i32;
            let oy = (style as i32) * FRAME_SIZE as i32;
            draw_plane_frame(&mut img, style, frame, ox, oy);
        }
    }

    img.save("assets/planes.png")
        .map_err(|e| format!("save failed: {e}"))?;
    println!("wrote assets/planes.png ({}x{})", width, height);
    Ok(())
}
