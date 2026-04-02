use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct FogLayerId(pub u8);

impl FogLayerId {
    pub const ZERO: Self = Self(0);
    pub const MAX_INDEX: u8 = 63;

    pub fn bit(self) -> u64 {
        assert!(
            self.0 <= Self::MAX_INDEX,
            "FogLayerId supports layer indices in the range 0..=63"
        );
        1_u64 << self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct FogLayerMask(pub u64);

impl FogLayerMask {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(u64::MAX);

    pub fn bit(layer: FogLayerId) -> Self {
        Self(layer.bit())
    }

    pub fn contains(self, layer: FogLayerId) -> bool {
        self.0 & layer.bit() != 0
    }

    pub fn insert(&mut self, layer: FogLayerId) {
        self.0 |= layer.bit();
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Default for FogLayerMask {
    fn default() -> Self {
        Self::EMPTY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum FogVisibilityState {
    Hidden,
    Explored,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct FogChunkCoord(pub UVec2);

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct FogGridSpec {
    pub origin: Vec2,
    pub dimensions: UVec2,
    pub cell_size: Vec2,
    pub chunk_size: UVec2,
}

impl FogGridSpec {
    pub fn new(origin: Vec2, dimensions: UVec2, cell_size: Vec2) -> Self {
        Self {
            origin,
            dimensions,
            cell_size: cell_size.max(Vec2::splat(0.001)),
            chunk_size: UVec2::splat(16),
        }
    }

    pub fn cell_count(self) -> usize {
        self.dimensions.x as usize * self.dimensions.y as usize
    }

    pub fn world_size(self) -> Vec2 {
        self.cell_size * self.dimensions.as_vec2()
    }

    pub fn chunk_count(self) -> UVec2 {
        UVec2::new(
            self.dimensions.x.div_ceil(self.chunk_size.x.max(1)),
            self.dimensions.y.div_ceil(self.chunk_size.y.max(1)),
        )
    }

    pub fn contains_cell(self, cell: IVec2) -> bool {
        cell.x >= 0
            && cell.y >= 0
            && cell.x < self.dimensions.x as i32
            && cell.y < self.dimensions.y as i32
    }

    pub fn world_to_cell(self, world: Vec2) -> Option<IVec2> {
        let local = world - self.origin;
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }

        let cell = IVec2::new(
            (local.x / self.cell_size.x).floor() as i32,
            (local.y / self.cell_size.y).floor() as i32,
        );
        self.contains_cell(cell).then_some(cell)
    }

    pub fn cell_to_world_center(self, cell: IVec2) -> Option<Vec2> {
        self.contains_cell(cell)
            .then_some(self.origin + (cell.as_vec2() + Vec2::splat(0.5)) * self.cell_size)
    }

    pub fn index(self, cell: IVec2) -> Option<usize> {
        self.contains_cell(cell)
            .then_some(cell.y as usize * self.dimensions.x as usize + cell.x as usize)
    }

    pub fn cell_from_index(self, index: usize) -> IVec2 {
        let width = self.dimensions.x as usize;
        IVec2::new((index % width) as i32, (index / width) as i32)
    }

    pub fn chunk_for_cell(self, cell: IVec2) -> Option<FogChunkCoord> {
        self.contains_cell(cell).then_some(FogChunkCoord(UVec2::new(
            cell.x as u32 / self.chunk_size.x.max(1),
            cell.y as u32 / self.chunk_size.y.max(1),
        )))
    }

    pub fn chunk_bounds(self, chunk: FogChunkCoord) -> IRect {
        let min = IVec2::new(
            (chunk.0.x * self.chunk_size.x) as i32,
            (chunk.0.y * self.chunk_size.y) as i32,
        );
        let max = IVec2::new(
            ((chunk.0.x + 1) * self.chunk_size.x).min(self.dimensions.x) as i32,
            ((chunk.0.y + 1) * self.chunk_size.y).min(self.dimensions.y) as i32,
        );
        IRect::from_corners(min, max)
    }
}

impl Default for FogGridSpec {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            dimensions: UVec2::new(64, 64),
            cell_size: Vec2::splat(1.0),
            chunk_size: UVec2::splat(16),
        }
    }
}

#[cfg(test)]
#[path = "grid_tests.rs"]
mod tests;
