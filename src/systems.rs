use bevy::platform::time::Instant;

use bevy::prelude::*;

use crate::{
    components::{VisionCellSource, VisionOccluder, VisionSource},
    messages::VisibilityMapUpdated,
    persistence::FogCustomPersistence,
    resources::{FogOfWarConfig, FogOfWarMap, FogOfWarStats, FogWorldAxes},
    visibility::{self, VisionCellSample, VisionOccluderSample, VisionSourceSample},
};

#[derive(Resource, Default)]
pub(crate) struct FogRuntimeState {
    pub active: bool,
    pub last_raster_micros: u64,
}

#[derive(Resource, Default)]
pub(crate) struct FogCollectedInputs {
    pub sources: Vec<VisionSourceSample>,
    pub cell_sources: Vec<VisionCellSample>,
    pub occluders: Vec<VisionOccluderSample>,
}

pub(crate) fn activate_runtime(mut runtime: ResMut<FogRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(
    mut runtime: ResMut<FogRuntimeState>,
    config: Res<FogOfWarConfig>,
    mut map: ResMut<FogOfWarMap>,
    custom_persistence: Option<Res<FogCustomPersistence>>,
    mut stats: ResMut<FogOfWarStats>,
    mut writer: MessageWriter<VisibilityMapUpdated>,
) {
    runtime.active = false;
    map.clear_blockers();
    map.clear_visible_counts();
    let updates =
        visibility::commit_visibility(&mut map, &config, custom_persistence.as_deref(), true);

    let (layer_count, visible_cells_total, explored_cells_total) = map.totals();
    stats.source_count = 0;
    stats.occluder_count = 0;
    stats.layer_count = layer_count;
    stats.dirty_chunk_count = updates.iter().map(|update| update.dirty_chunks.len()).sum();
    stats.visible_cells_total = visible_cells_total;
    stats.explored_cells_total = explored_cells_total;

    for update in updates {
        writer.write(update);
    }
}

pub(crate) fn runtime_is_active(runtime: Res<FogRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn collect_inputs(
    config: Res<FogOfWarConfig>,
    mut map: ResMut<FogOfWarMap>,
    mut stats: ResMut<FogOfWarStats>,
    mut collected: ResMut<FogCollectedInputs>,
    sources: Query<(&GlobalTransform, &VisionSource)>,
    cell_sources: Query<&VisionCellSource>,
    occluders: Query<(&GlobalTransform, &VisionOccluder)>,
) {
    map.reconfigure(config.clone());

    collected.sources.clear();
    collected.cell_sources.clear();
    collected.occluders.clear();
    let mut enabled_sources = 0;
    let mut enabled_cell_sources = 0;
    let mut enabled_occluders = 0;

    for (transform, source) in &sources {
        if !source.enabled {
            continue;
        }
        enabled_sources += 1;
        for layer in source.resolved_layers().iter_layers() {
            collected.sources.push(VisionSourceSample {
                layer,
                position: projected_position(transform, &config) + source.offset,
                shape: source.shape,
            });
        }
    }

    for cell_source in &cell_sources {
        if !cell_source.enabled {
            continue;
        }
        enabled_cell_sources += 1;
        for layer in cell_source.layers.iter_layers() {
            for cell in &cell_source.cells {
                collected
                    .cell_sources
                    .push(VisionCellSample { layer, cell: *cell });
            }
        }
    }

    for (transform, occluder) in &occluders {
        if !occluder.enabled {
            continue;
        }
        enabled_occluders += 1;
        collected.occluders.push(VisionOccluderSample {
            layers: occluder.layers,
            position: projected_position(transform, &config) + occluder.offset,
            shape: occluder.shape,
        });
    }

    stats.source_count = enabled_sources + enabled_cell_sources;
    stats.occluder_count = enabled_occluders;
}

pub(crate) fn compute_visibility(
    mut runtime: ResMut<FogRuntimeState>,
    collected: Res<FogCollectedInputs>,
    mut map: ResMut<FogOfWarMap>,
) {
    let start = Instant::now();
    visibility::rebuild_blockers(&mut map, &collected.occluders);
    visibility::accumulate_visibility(&mut map, &collected.sources, &collected.cell_sources);
    runtime.last_raster_micros = start.elapsed().as_micros() as u64;
}

pub(crate) fn apply_persistence(
    runtime: Res<FogRuntimeState>,
    config: Res<FogOfWarConfig>,
    mut map: ResMut<FogOfWarMap>,
    custom_persistence: Option<Res<FogCustomPersistence>>,
    mut stats: ResMut<FogOfWarStats>,
    mut writer: MessageWriter<VisibilityMapUpdated>,
) {
    let start = Instant::now();
    let updates =
        visibility::commit_visibility(&mut map, &config, custom_persistence.as_deref(), false);
    let commit_micros = start.elapsed().as_micros() as u64;
    let (layer_count, visible_cells_total, explored_cells_total) = map.totals();

    stats.last_compute_micros = runtime.last_raster_micros + commit_micros;
    stats.layer_count = layer_count;
    stats.dirty_chunk_count = updates.iter().map(|update| update.dirty_chunks.len()).sum();
    stats.visible_cells_total = visible_cells_total;
    stats.explored_cells_total = explored_cells_total;

    for update in updates {
        writer.write(update);
    }
}

fn projected_position(transform: &GlobalTransform, config: &FogOfWarConfig) -> Vec2 {
    let translation = transform.translation();
    match config.world_axes {
        FogWorldAxes::XY => translation.truncate(),
        FogWorldAxes::XZ => Vec2::new(translation.x, translation.z),
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod tests;
