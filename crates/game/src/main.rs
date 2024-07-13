use game::*;
use macroquad::prelude::*;

#[macroquad::main("Cells")]
async fn main() {
    let screen_width = screen_width() as u16;
    let screen_height = screen_height() as u16;

    let mut state = GameState::new(screen_width, screen_height);
    let sandbox_size = state.sandbox.size();

    let mut image = Image::gen_image_color(sandbox_size.width, sandbox_size.height, BLACK);
    let texture = Texture2D::from_image(&image);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        state.on_frame();

        state.sandbox.draw_to_image(&mut image);

        clear_background(WHITE);

        texture.update(&image);
        draw_texture_ex(
            &texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: vec2(
                    (sandbox_size.width * state.pixels_per_cell) as f32,
                    (sandbox_size.height * state.pixels_per_cell) as f32,
                )
                .into(),
                ..Default::default()
            },
        );
        state.draw_debug_text();

        next_frame().await;
    }
}
