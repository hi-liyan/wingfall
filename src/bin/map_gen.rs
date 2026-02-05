use image::{ImageBuffer, Rgba, RgbaImage};

const WIDTH: u32 = 480;
const HEIGHT: u32 = 720;

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

fn fill_rect(img: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            set_px(img, x, y, color);
        }
    }
}

fn fill_circle(img: &mut RgbaImage, cx: i32, cy: i32, r: i32, color: Color) {
    let r2 = (r * r) as i32;
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r2 {
                set_px(img, x, y, color);
            }
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

fn draw_starfield(img: &mut RgbaImage, seed: u32, count: usize) {
    for i in 0..count {
        let s = seed ^ (i as u32).wrapping_mul(2654435761);
        let x = (rand_f32(s) * WIDTH as f32) as i32;
        let y = (rand_f32(s ^ 0x55aa) * HEIGHT as f32) as i32;
        let size = 1 + (rand_f32(s ^ 0xa5a5) * 2.0) as i32;
        let bright = (180.0 + rand_f32(s ^ 0x1111) * 75.0) as u8;
        let color = Color::rgba(bright, bright, 255, 220);
        fill_circle(img, x, y, size, color);
        if size >= 2 {
            set_px(img, x + 2, y, Color::rgba(120, 160, 255, 120));
            set_px(img, x - 2, y, Color::rgba(120, 160, 255, 120));
        }
    }
}

fn draw_nebula(img: &mut RgbaImage, seed: u32, center: (i32, i32), r: i32, color: Color) {
    let (cx, cy) = center;
    for i in 0..200 {
        let s = seed ^ (i as u32 * 7919);
        let ang = rand_f32(s) * std::f32::consts::TAU;
        let dist = rand_f32(s ^ 0x2222) * r as f32;
        let x = cx as f32 + ang.cos() * dist;
        let y = cy as f32 + ang.sin() * dist * 0.8;
        let radius = 18.0 + rand_f32(s ^ 0x3333) * 26.0;
        let alpha = (color.a as f32 * (1.0 - dist / r as f32)).max(0.0) as u8;
        let mut c = color;
        c.a = alpha;
        fill_circle(img, x as i32, y as i32, radius as i32, c);
    }
}

fn draw_planet(img: &mut RgbaImage, center: (i32, i32), r: i32) {
    let (cx, cy) = center;
    fill_circle(img, cx, cy, r, Color::rgba(35, 60, 90, 255));
    fill_circle(img, cx - 8, cy - 6, r - 8, Color::rgba(25, 45, 70, 255));
    fill_circle(img, cx + 6, cy + 8, r - 18, Color::rgba(20, 35, 55, 255));
    fill_circle(img, cx, cy, r + 18, Color::rgba(80, 140, 200, 40));
}

fn main() -> Result<(), String> {
    let mut img: RgbaImage = ImageBuffer::new(WIDTH, HEIGHT);

    fill_rect(
        &mut img,
        0,
        0,
        WIDTH as i32 - 1,
        HEIGHT as i32 - 1,
        Color::rgba(8, 10, 18, 255),
    );

    draw_nebula(
        &mut img,
        0x1234,
        (WIDTH as i32 / 2, HEIGHT as i32 / 3),
        220,
        Color::rgba(60, 80, 140, 70),
    );
    draw_nebula(
        &mut img,
        0x5678,
        (WIDTH as i32 / 3, HEIGHT as i32 / 2),
        180,
        Color::rgba(120, 70, 160, 60),
    );
    draw_nebula(
        &mut img,
        0x9abc,
        (WIDTH as i32 * 2 / 3, HEIGHT as i32 / 2),
        200,
        Color::rgba(70, 120, 160, 55),
    );

    draw_planet(&mut img, (WIDTH as i32 - 80, 120), 60);
    draw_starfield(&mut img, 0xbeef, 260);

    img.save("assets/map_space.png")
        .map_err(|e| format!("save failed: {e}"))?;
    println!("wrote assets/map_space.png ({}x{})", WIDTH, HEIGHT);
    Ok(())
}
