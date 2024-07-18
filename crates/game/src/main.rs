use game::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Cells".to_owned(),
        high_dpi: true,

        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let dpi_scale = screen_dpi_scale();
    let width = (screen_width() * dpi_scale) as u16;
    let height = (screen_height() * dpi_scale) as u16;

    println!("Screen size: {}x{}", width, height);

    let mut state = GameState::new();

    let max_chunk = CellPosition::new(width as i32 - 1, height as i32 - 1).get_chunk_position();
    for x in 0..=max_chunk.x {
        for y in 0..=max_chunk.y {
            state.world.ensure_chunk(ChunkPosition::new(x, y));
        }
    }

    let mut image = Image::gen_image_color(width, height, BLACK);
    let texture = Texture2D::from_image(&image);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        state.on_frame();

        state.draw_to_image(&mut image);

        clear_background(WHITE);

        texture.update(&image);
        draw_texture_ex(
            &texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: vec2(screen_width(), screen_height()).into(),
                ..Default::default()
            },
        );
        state.draw_debug_text();

        next_frame().await;
    }
}
