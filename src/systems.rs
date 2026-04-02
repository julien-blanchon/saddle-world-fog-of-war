use bevy::platform::time::Instant;

use bevy::prelude::*;

use crate::{
    components::{VisionOccluder, VisionSource},
    messages::VisibilityMapUpdated,
    resources::{FogOfWarConfig, FogOfWarMap, FogOfWarStats, FogWorldAxes},
    visibility::{self, VisionOccluderSample, VisionSourceSample},
};

#[derive(Resource, Default)]
pub(crate) struct FogRuntimeState {
    pub active: bool,
    pub last_raster_micros: u64,
}

#[derive(Resource, Default)]
pub(crate) struct FogCollectedInputs {
    pub sources: Vec<VisionSourceSample>,
    pub occluders: Vec<VisionOccluderSample>,
}

pub(crate) fn activate_runtime(mut runtime: ResMut<FogRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(
    mut runtime: ResMut<FogRuntimeState>,
    mut map: ResMut<FogOfWarMap>,
    mut stats: ResMut<FogOfWarStats>,
) {
    runtime.active = false;
    map.deactivate();

    let (layer_count, visible_cells_total, explored_cells_total) = map.totals();
    stats.source_count = 0;
    stats.occluder_count = 0;
    stats.layer_count = layer_count;
    stats.dirty_chunk_count = 0;
    stats.visible_cells_total = visible_cells_total;
    stats.explored_cells_total = explored_cells_total;
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
    occluders: Query<(&GlobalTransform, &VisionOccluder)>,
) {
    map.reconfigure(config.clone());

    collected.sources.clear();
    collected.occluders.clear();

    for (transform, source) in &sources {
        if !source.enabled {
            continue;
        }
        collected.sources.push(VisionSourceSample {
            layer: source.layer,
            position: projected_position(transform, &config) + source.offset,
            shape: source.shape,
        });
    }

    for (transform, occluder) in &occluders {
        if !occluder.enabled {
            continue;
        }
        collected.occluders.push(VisionOccluderSample {
            layers: occluder.layers,
            position: projected_position(transform, &config) + occluder.offset,
            shape: occluder.shape,
        });
    }

    stats.source_count = collected.sources.len();
    stats.occluder_count = collected.occluders.len();
}

pub(crate) fn compute_visibility(
    mut runtime: ResMut<FogRuntimeState>,
    collected: Res<FogCollectedInputs>,
    mut map: ResMut<FogOfWarMap>,
) {
    let start = Instant::now();
    visibility::rebuild_blockers(&mut map, &collected.occluders);
    visibility::accumulate_visibility(&mut map, &collected.sources);
    runtime.last_raster_micros = start.elapsed().as_micros() as u64;
}

pub(crate) fn update_exploration_memory(
    runtime: Res<FogRuntimeState>,
    mut map: ResMut<FogOfWarMap>,
    mut stats: ResMut<FogOfWarStats>,
    mut writer: MessageWriter<VisibilityMapUpdated>,
) {
    let start = Instant::now();
    let updates = visibility::commit_visibility(&mut map);
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
