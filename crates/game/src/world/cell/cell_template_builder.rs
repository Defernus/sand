use crate::*;
use eyre::ContextCompat;
use nohash_hasher::IntMap;
use std::collections::HashMap;

pub struct CellTemplateBuilder {
    pub next_id: CellId,
    pub id_by_label: HashMap<String, CellId>,
    pub cells: IntMap<CellId, CellMeta>,
}

impl CellTemplateBuilder {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            id_by_label: Default::default(),
            cells: Default::default(),
        }
    }

    pub fn add_cell(&mut self, mut cell_meta: CellMeta) -> CellId {
        let id = self.ensure_id_by_label(cell_meta.label.clone());
        cell_meta.id = id;
        self.cells.insert(id, cell_meta);
        id
    }

    /// Ensure that cell with given label will have own unique id.
    pub fn ensure_id_by_label(&mut self, label: impl Into<String>) -> CellId {
        *self.id_by_label.entry(label.into()).or_insert_with(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        })
    }

    pub fn build(mut self) -> eyre::Result<CellsTemplate> {
        let cells_amount = self.cells.len();
        let mut cells: Vec<CellMeta> = Vec::with_capacity(self.cells.len());

        for id in 0..cells_amount as CellId {
            let cell = self.cells.remove(&id).context("Cell id not found")?;
            cells.push(cell);
        }

        Ok(CellsTemplate { cells })
    }
}
