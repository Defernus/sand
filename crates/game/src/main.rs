use game::*;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Cells".to_owned(),

        ..Default::default()
    }
}

fn gen_world(world: &mut WorldState) {
    let floor_height = 3;
    let floor_width: i32 = 200;
    for x in -(floor_width / 2)..(floor_width / 2) {
        for y in 0..floor_height {
            let pos = GlobalCellPos::new(x, y);
            world.set_cell(pos, CELL_STONE.init());
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::new();

    gen_world(&mut state.world);

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
