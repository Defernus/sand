use crate::*;

pub fn gen_world(world: &mut WorldState, cells_template: &CellsTemplate) {
    let sand_meta = cells_template.get_cell_meta_by_label("Sand").unwrap();
    let stone_meta = cells_template.get_cell_meta_by_label("Stone").unwrap();

    gen_rect(
        world,
        cells_template,
        GlobalCellPos::new(-1500, 0),
        GlobalCellPos::new(1500, 3),
        stone_meta,
    );

    gen_rect(
        world,
        cells_template,
        GlobalCellPos::new(200, 10),
        GlobalCellPos::new(300, 510),
        sand_meta,
    );
}

pub fn gen_rect(
    world: &mut WorldState,
    cells_template: &CellsTemplate,
    start: GlobalCellPos,
    end: GlobalCellPos,
    cell_meta: &CellMeta,
) {
    for y in start.y()..end.y() {
        for x in start.x()..end.x() {
            let pos = GlobalCellPos::new(x, y);
            world.set_cell(pos, cell_meta.init(), cells_template);
        }
    }
}
