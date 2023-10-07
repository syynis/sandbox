use bevy::math::*;

pub struct Aabb {
    min: UVec2,
    max: UVec2,
}

impl Aabb {
    pub fn contains(&self, pos: UVec2) -> bool {
        (self.min.x..self.max.x).contains(&pos.x) && (self.min.y..self.max.y).contains(&pos.y)
    }
}

pub struct AabbCollection {
    aabbs: Vec<Aabb>,
    size: UVec2,
}

impl AabbCollection {
    pub fn new(size: UVec2) -> Self {
        Self {
            aabbs: Vec::new(),
            size,
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn get(&self, pos: UVec2) -> Option<&Aabb> {
        let idx = self.get_idx(pos)?;
        self.aabbs.get(idx)
    }

    pub fn get_mut(&mut self, pos: UVec2) -> Option<&mut Aabb> {
        let idx = self.get_idx(pos)?;
        self.aabbs.get_mut(idx)
    }

    pub fn get_idx(&self, pos: UVec2) -> Option<usize> {
        self.aabbs
            .iter()
            .enumerate()
            .find(|(_, aabb)| aabb.contains(pos))
            .map(|e| e.0)
    }

    pub fn remove(&mut self, pos: UVec2) -> Option<Aabb> {
        let idx = self.get_idx(pos)?;

        Some(self.aabbs.swap_remove(idx))
    }

    pub fn add(&mut self, pos: UVec2, extents: UVec2) {
        if self.get(pos).is_none() {
            self.aabbs.push(Aabb {
                min: pos,
                max: pos + extents,
            })
        }
    }
}
