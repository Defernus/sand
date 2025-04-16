use game::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Cells".to_owned(),

        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::new();

    gen_world(&mut state.world, &state.cells_template);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        state.on_frame();

        clear_background(BLACK);

        state.draw_to_screen();
        state.draw_debug_text();

        next_frame().await;
    }
}
