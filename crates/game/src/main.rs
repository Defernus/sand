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
    for x in -100..100 as i32 {
        for y in 0..floor_height {
            let pos = GlobalCellPos::new(x, y);
            world.set_cell(pos, Cell::new(CELL_STONE));
        }
    }

    for y in (floor_height + 1)..100 {
        let pos = GlobalCellPos::new(0, y);

        world.set_cell(pos, Cell::new(CELL_SAND));
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
