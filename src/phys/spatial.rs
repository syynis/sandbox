use bevy::prelude::*;
use bevy::utils::hashbrown::*;

#[derive(Default, Debug, Clone)]
pub struct SpatialGrid<const LG_CELL_SZ: usize, const LG_COARSE_CELL_SZ: usize> {
    grid: HashMap<IVec2, Vec<Entity>>,
    coarse_grid: HashMap<IVec2, Vec<Entity>>,

    coarse_radius_threshold: u32,
    maximum_radius: u32,
}

#[derive(Clone)]
pub struct Aabr {
    min: IVec2,
    max: IVec2,
}

impl<const LG_CELL_SZ: usize, const LG_COARSE_CELL_SZ: usize>
    SpatialGrid<LG_CELL_SZ, LG_COARSE_CELL_SZ>
{
    pub fn new(coarse_radius_threshold: u32) -> Self {
        Self {
            coarse_radius_threshold,
            maximum_radius: coarse_radius_threshold,
            ..default()
        }
    }

    pub fn insert(&mut self, pos: IVec2, entity_radius: u32, entity: Entity) {
        if entity_radius <= self.coarse_radius_threshold {
            let cell = IVec2::new(pos.x >> LG_CELL_SZ, pos.y >> LG_CELL_SZ);
            self.grid.entry(cell).or_default().push(entity);
        } else {
            let cell = IVec2::new(pos.x >> LG_COARSE_CELL_SZ, pos.y >> LG_COARSE_CELL_SZ);
            self.coarse_grid.entry(cell).or_default().push(entity);
            self.maximum_radius = self.maximum_radius.max(entity_radius);
        }
    }

    pub fn iter_area<'a>(&'a self, area: Aabr) -> impl Iterator<Item = Entity> + 'a {
        let iter = |max_entity_radius, grid: &'a HashMap<IVec2, Vec<Entity>>, lg_cell_size| {
            let min = area.min - max_entity_radius as i32;
            let max = area.max + max_entity_radius as i32;
            let min = IVec2::new(min.x >> lg_cell_size, min.y >> lg_cell_size);
            let max = IVec2::new(
                (max.x + (1 << lg_cell_size) - 1) >> lg_cell_size,
                (max.y + (1 << lg_cell_size) - 1) >> lg_cell_size,
            );

            (min.x..=max.x)
                .map(move |x| (min.y..=max.y).map(move |y| IVec2::new(x, y)))
                .flatten()
                .filter_map(move |pos| grid.get(&pos))
                .flatten()
                .copied()
        };

        iter(self.coarse_radius_threshold, &self.grid, LG_CELL_SZ).chain(iter(
            self.maximum_radius,
            &self.coarse_grid,
            LG_COARSE_CELL_SZ,
        ))
    }

    pub fn clear(&mut self) {
        self.grid.clear();
        self.coarse_grid.clear();
        self.maximum_radius = self.coarse_radius_threshold;
    }
}
