use bevy::prelude::*;

use crate::grid::FogLayerId;

#[derive(Message, Clone, Debug)]
pub struct VisibilityMapUpdated {
    pub layer: FogLayerId,
    pub visible_cells: usize,
    pub explored_cells: usize,
    pub dirty_chunks: Vec<UVec2>,
}
