use macroquad::prelude::*;

pub fn draw_text_shadow(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let shadow_alpha = 0.5;
    let shadow_offset_x = 1.0;
    let shadow_offset_y = 1.0;

    draw_text(
        text,
        x + shadow_offset_x,
        y + shadow_offset_y,
        font_size,
        Color::new(0.0, 0.0, 0.0, shadow_alpha),
    );
    draw_text(text, x, y, font_size, color);
}
