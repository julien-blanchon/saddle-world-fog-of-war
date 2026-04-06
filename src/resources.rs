use std::collections::{HashMap, HashSet};

use bevy::{asset::Handle, image::Image, prelude::*};

use crate::grid::{FogGridSpec, FogLayerId, FogLayerMask, FogVisibilityState};
use crate::persistence::FogPersistenceMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FogOcclusionMode {
    Disabled,
    Bresenham,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FogWorldAxes {
    XY,
    XZ,
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct FogOfWarConfig {
    pub grid: FogGridSpec,
    pub occlusion_mode: FogOcclusionMode,
    pub world_axes: FogWorldAxes,
    pub persistence_mode: FogPersistenceMode,
}

impl Default for FogOfWarConfig {
    fn default() -> Self {
        Self {
            grid: FogGridSpec::default(),
            occlusion_mode: FogOcclusionMode::Bresenham,
            world_axes: FogWorldAxes::XY,
            persistence_mode: FogPersistenceMode::ExploredMemory,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct FogLayerSummary {
    pub visible_cells: usize,
    pub explored_cells: usize,
}

#[derive(Default, Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct FogOfWarStats {
    pub last_compute_micros: u64,
    pub last_upload_micros: u64,
    pub source_count: usize,
    pub occluder_count: usize,
    pub layer_count: usize,
    pub dirty_chunk_count: usize,
    pub visible_cells_total: usize,
    pub explored_cells_total: usize,
}

#[derive(Default, Resource, Debug, Clone)]
pub struct FogOfWarRenderAssets {
    pub images: HashMap<FogLayerId, Handle<Image>>,
}

impl FogOfWarRenderAssets {
    pub fn image(&self, layer: FogLayerId) -> Option<Handle<Image>> {
        self.images.get(&layer).cloned()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FogLayerState {
    pub states: Vec<FogVisibilityState>,
    pub visible_now: Vec<bool>,
    pub visible_counts: Vec<u16>,
    pub dirty_chunks: HashSet<UVec2>,
    pub visible_cells: usize,
    pub explored_cells: usize,
}

impl FogLayerState {
    fn new(cell_count: usize) -> Self {
        Self {
            states: vec![FogVisibilityState::Hidden; cell_count],
            visible_now: vec![false; cell_count],
            visible_counts: vec![0; cell_count],
            dirty_chunks: HashSet::new(),
            visible_cells: 0,
            explored_cells: 0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct FogOfWarMap {
    config: FogOfWarConfig,
    blockers: Vec<FogLayerMask>,
    layers: HashMap<FogLayerId, FogLayerState>,
}

impl FogOfWarMap {
    pub fn new(config: FogOfWarConfig) -> Self {
        let cell_count = config.grid.cell_count();
        Self {
            blockers: vec![FogLayerMask::EMPTY; cell_count],
            layers: HashMap::default(),
            config,
        }
    }

    pub fn config(&self) -> &FogOfWarConfig {
        &self.config
    }

    pub fn grid(&self) -> FogGridSpec {
        self.config.grid
    }

    pub fn layer_ids(&self) -> impl Iterator<Item = FogLayerId> + '_ {
        self.layers.keys().copied()
    }

    pub fn visibility_at_cell(&self, layer: FogLayerId, cell: IVec2) -> Option<FogVisibilityState> {
        let index = self.config.grid.index(cell)?;
        self.layers
            .get(&layer)
            .map(|state| state.states[index])
            .or(Some(FogVisibilityState::Hidden))
    }

    pub fn current_visibility_at_cell(&self, layer: FogLayerId, cell: IVec2) -> Option<bool> {
        let index = self.config.grid.index(cell)?;
        self.layers
            .get(&layer)
            .map(|state| state.visible_now[index])
            .or(Some(false))
    }

    pub fn visibility_at_world_pos(
        &self,
        layer: FogLayerId,
        world: Vec2,
    ) -> Option<FogVisibilityState> {
        let cell = self.config.grid.world_to_cell(world)?;
        self.visibility_at_cell(layer, cell)
    }

    pub fn is_visible(&self, layer: FogLayerId, cell: IVec2) -> bool {
        self.current_visibility_at_cell(layer, cell) == Some(true)
    }

    pub fn is_explored(&self, layer: FogLayerId, cell: IVec2) -> bool {
        self.visibility_at_cell(layer, cell)
            .is_some_and(|state| state != FogVisibilityState::Hidden)
    }

    pub fn iter_visible_cells(&self, layer: FogLayerId) -> impl Iterator<Item = IVec2> + '_ {
        let grid = self.config.grid;
        self.layers.get(&layer).into_iter().flat_map(move |state| {
            state
                .visible_now
                .iter()
                .enumerate()
                .filter_map(move |(index, cell)| (*cell).then_some(grid.cell_from_index(index)))
        })
    }

    pub fn iter_explored_cells(&self, layer: FogLayerId) -> impl Iterator<Item = IVec2> + '_ {
        let grid = self.config.grid;
        self.layers.get(&layer).into_iter().flat_map(move |state| {
            state
                .states
                .iter()
                .enumerate()
                .filter_map(move |(index, cell)| {
                    (*cell != FogVisibilityState::Hidden).then_some(grid.cell_from_index(index))
                })
        })
    }

    pub fn layer_summary(&self, layer: FogLayerId) -> Option<FogLayerSummary> {
        self.layers.get(&layer).map(|state| FogLayerSummary {
            visible_cells: state.visible_cells,
            explored_cells: state.explored_cells,
        })
    }

    pub fn blocker_at_cell(&self, layer: FogLayerId, cell: IVec2) -> bool {
        self.config
            .grid
            .index(cell)
            .and_then(|index| self.blockers.get(index).copied())
            .is_some_and(|mask| mask.contains(layer))
    }

    pub(crate) fn reconfigure(&mut self, config: FogOfWarConfig) {
        if self.config == config {
            return;
        }

        *self = Self::new(config);
    }

    pub(crate) fn clear_blockers(&mut self) {
        self.blockers.fill(FogLayerMask::EMPTY);
    }

    pub(crate) fn mark_blocker(&mut self, cell: IVec2, layers: FogLayerMask) {
        let Some(index) = self.config.grid.index(cell) else {
            return;
        };
        self.blockers[index] = self.blockers[index].union(layers);
    }

    pub(crate) fn clear_visible_counts(&mut self) {
        for layer in self.layers.values_mut() {
            layer.visible_counts.fill(0);
        }
    }

    pub(crate) fn ensure_layer_mut(&mut self, layer: FogLayerId) -> &mut FogLayerState {
        let cell_count = self.config.grid.cell_count();
        self.layers
            .entry(layer)
            .or_insert_with(|| FogLayerState::new(cell_count))
    }

    pub(crate) fn mark_visible(&mut self, layer: FogLayerId, cell: IVec2) {
        let Some(index) = self.config.grid.index(cell) else {
            return;
        };

        let layer_state = self.ensure_layer_mut(layer);
        layer_state.visible_counts[index] = layer_state.visible_counts[index].saturating_add(1);
    }

    pub(crate) fn layers_mut(&mut self) -> impl Iterator<Item = (FogLayerId, &mut FogLayerState)> {
        self.layers.iter_mut().map(|(layer, state)| (*layer, state))
    }

    pub(crate) fn layers(&self) -> impl Iterator<Item = (FogLayerId, &FogLayerState)> {
        self.layers.iter().map(|(layer, state)| (*layer, state))
    }

    pub(crate) fn totals(&self) -> (usize, usize, usize) {
        self.layers.values().fold(
            (0, 0, 0),
            |(layer_count, visible_cells, explored_cells), layer| {
                (
                    layer_count + 1,
                    visible_cells + layer.visible_cells,
                    explored_cells + layer.explored_cells,
                )
            },
        )
    }

    pub(crate) fn state_byte(state: FogVisibilityState) -> u8 {
        match state {
            FogVisibilityState::Hidden => 0,
            FogVisibilityState::Explored => 127,
            FogVisibilityState::Visible => 255,
        }
    }
}

impl Default for FogOfWarMap {
    fn default() -> Self {
        Self::new(FogOfWarConfig::default())
    }
}
