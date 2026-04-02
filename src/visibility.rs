use bevy::prelude::*;

use crate::{
    components::{FogOccluderShape, FogRevealShape},
    grid::{FogLayerMask, FogVisibilityState},
    math::{bresenham_line, safe_normalize_or},
    messages::VisibilityMapUpdated,
    resources::FogOfWarMap,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct VisionSourceSample {
    pub layer: crate::grid::FogLayerId,
    pub position: Vec2,
    pub shape: FogRevealShape,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VisionOccluderSample {
    pub layers: FogLayerMask,
    pub position: Vec2,
    pub shape: FogOccluderShape,
}

pub(crate) fn rebuild_blockers(map: &mut FogOfWarMap, occluders: &[VisionOccluderSample]) {
    map.clear_blockers();
    let grid = map.grid();

    for occluder in occluders {
        match occluder.shape {
            FogOccluderShape::Cell => {
                if let Some(cell) = grid.world_to_cell(occluder.position) {
                    map.mark_blocker(cell, occluder.layers);
                }
            }
            _ => {
                let half_extents = occluder_half_extents(occluder.shape);
                let min = occluder.position - half_extents;
                let max = occluder.position + half_extents;
                let Some(min_cell) = grid.world_to_cell(min.max(grid.origin)) else {
                    continue;
                };
                let max_world = (max - Vec2::splat(0.001))
                    .min(grid.origin + grid.world_size() - Vec2::splat(0.001));
                let Some(max_cell) = grid.world_to_cell(max_world) else {
                    continue;
                };

                for y in min_cell.y..=max_cell.y {
                    for x in min_cell.x..=max_cell.x {
                        let cell = IVec2::new(x, y);
                        let Some(center) = grid.cell_to_world_center(cell) else {
                            continue;
                        };
                        if occluder_contains(occluder.shape, center - occluder.position) {
                            map.mark_blocker(cell, occluder.layers);
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn accumulate_visibility(map: &mut FogOfWarMap, sources: &[VisionSourceSample]) {
    map.clear_visible_counts();
    let grid = map.grid();

    for source in sources {
        let Some(origin_cell) = grid.world_to_cell(source.position) else {
            continue;
        };
        let _ = map.ensure_layer_mut(source.layer);
        let half_extents = reveal_half_extents(source.shape);
        let min_world = source.position - half_extents;
        let max_world = source.position + half_extents;

        let clamped_min = min_world.max(grid.origin);
        let clamped_max = (max_world - Vec2::splat(0.001))
            .min(grid.origin + grid.world_size() - Vec2::splat(0.001));

        let Some(min_cell) = grid.world_to_cell(clamped_min) else {
            continue;
        };
        let Some(max_cell) = grid.world_to_cell(clamped_max) else {
            continue;
        };

        for y in min_cell.y..=max_cell.y {
            for x in min_cell.x..=max_cell.x {
                let candidate = IVec2::new(x, y);
                let Some(candidate_center) = grid.cell_to_world_center(candidate) else {
                    continue;
                };

                let local = candidate_center - source.position;
                if !reveal_contains(source.shape, local) {
                    continue;
                }

                if !has_line_of_sight(map, source.layer, origin_cell, candidate) {
                    continue;
                }

                map.mark_visible(source.layer, candidate);
            }
        }
    }
}

pub(crate) fn commit_visibility(map: &mut FogOfWarMap) -> Vec<VisibilityMapUpdated> {
    let grid = map.grid();
    let mut updates = Vec::new();

    for (layer_id, layer_state) in map.layers_mut() {
        layer_state.dirty_chunks.clear();
        let mut visible_cells = 0;
        let mut explored_cells = 0;

        for index in 0..layer_state.states.len() {
            let previous = layer_state.states[index];
            let next = if layer_state.visible_counts[index] > 0 {
                FogVisibilityState::Visible
            } else if previous == FogVisibilityState::Hidden {
                FogVisibilityState::Hidden
            } else {
                FogVisibilityState::Explored
            };

            if next != previous {
                if let Some(chunk) = grid.chunk_for_cell(grid.cell_from_index(index)) {
                    layer_state.dirty_chunks.insert(chunk.0);
                }
                layer_state.states[index] = next;
            }

            if next == FogVisibilityState::Visible {
                visible_cells += 1;
            }
            if next != FogVisibilityState::Hidden {
                explored_cells += 1;
            }
        }

        layer_state.visible_cells = visible_cells;
        layer_state.explored_cells = explored_cells;

        if !layer_state.dirty_chunks.is_empty() {
            let mut dirty_chunks: Vec<_> = layer_state.dirty_chunks.iter().copied().collect();
            dirty_chunks.sort_by_key(|chunk| (chunk.y, chunk.x));
            updates.push(VisibilityMapUpdated {
                layer: layer_id,
                visible_cells,
                explored_cells,
                dirty_chunks,
            });
        }
    }

    updates
}

fn reveal_half_extents(shape: FogRevealShape) -> Vec2 {
    match shape {
        FogRevealShape::Circle { radius } | FogRevealShape::Arc { radius, .. } => {
            Vec2::splat(radius.max(0.0))
        }
        FogRevealShape::Rect { half_extents } => half_extents.max(Vec2::ZERO),
    }
}

fn occluder_half_extents(shape: FogOccluderShape) -> Vec2 {
    match shape {
        FogOccluderShape::Cell => Vec2::ZERO,
        FogOccluderShape::Circle { radius } => Vec2::splat(radius.max(0.0)),
        FogOccluderShape::Rect { half_extents } => half_extents.max(Vec2::ZERO),
    }
}

fn reveal_contains(shape: FogRevealShape, local: Vec2) -> bool {
    match shape {
        FogRevealShape::Circle { radius } => local.length_squared() <= radius * radius,
        FogRevealShape::Arc {
            radius,
            angle_radians,
            facing,
        } => {
            if local.length_squared() > radius * radius {
                return false;
            }
            if local.length_squared() <= f32::EPSILON {
                return true;
            }
            let forward = safe_normalize_or(facing, Vec2::X);
            let direction = local.normalize();
            let half_angle = angle_radians * 0.5;
            direction.dot(forward) >= half_angle.cos()
        }
        FogRevealShape::Rect { half_extents } => {
            local.x.abs() <= half_extents.x && local.y.abs() <= half_extents.y
        }
    }
}

fn occluder_contains(shape: FogOccluderShape, local: Vec2) -> bool {
    match shape {
        FogOccluderShape::Cell => local.length_squared() <= 0.001,
        FogOccluderShape::Circle { radius } => local.length_squared() <= radius * radius,
        FogOccluderShape::Rect { half_extents } => {
            local.x.abs() <= half_extents.x && local.y.abs() <= half_extents.y
        }
    }
}

fn has_line_of_sight(
    map: &FogOfWarMap,
    layer: crate::grid::FogLayerId,
    origin: IVec2,
    target: IVec2,
) -> bool {
    if map.config().occlusion_mode == crate::resources::FogOcclusionMode::Disabled {
        return true;
    }

    let line = bresenham_line(origin, target);
    let last_index = line.len().saturating_sub(1);
    for (index, cell) in line.into_iter().enumerate().skip(1) {
        if index == last_index {
            return true;
        }
        if map.blocker_at_cell(layer, cell) {
            return false;
        }
    }

    true
}

#[cfg(test)]
#[path = "visibility_tests.rs"]
mod tests;
