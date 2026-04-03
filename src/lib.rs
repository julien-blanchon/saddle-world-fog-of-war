mod components;
mod grid;
mod math;
mod messages;
mod rendering;
mod resources;
mod systems;
mod visibility;

pub use components::{
    FogOccluderShape, FogOverlay2d, FogPalette, FogProjectionReceiver, FogRevealShape,
    VisionCellSource, VisionOccluder, VisionSource,
};
pub use grid::{FogChunkCoord, FogGridSpec, FogLayerId, FogLayerMask, FogVisibilityState};
pub use messages::VisibilityMapUpdated;
pub use resources::{
    FogLayerSummary, FogOcclusionMode, FogOfWarConfig, FogOfWarMap, FogOfWarRenderAssets,
    FogOfWarStats, FogWorldAxes,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FogOfWarSystems {
    CollectVisionSources,
    ComputeVisibility,
    UpdateExplorationMemory,
    UploadRenderData,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct FogOfWarPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub config: FogOfWarConfig,
}

impl FogOfWarPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
            config: FogOfWarConfig::default(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_config(mut self, config: FogOfWarConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for FogOfWarPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for FogOfWarPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.insert_resource(self.config.clone())
            .insert_resource(FogOfWarMap::new(self.config.clone()))
            .init_resource::<FogOfWarStats>()
            .init_resource::<FogOfWarRenderAssets>()
            .init_resource::<systems::FogCollectedInputs>()
            .init_resource::<systems::FogRuntimeState>()
            .add_message::<VisibilityMapUpdated>()
            .register_type::<FogChunkCoord>()
            .register_type::<FogGridSpec>()
            .register_type::<FogLayerId>()
            .register_type::<FogLayerMask>()
            .register_type::<FogLayerSummary>()
            .register_type::<FogOccluderShape>()
            .register_type::<FogOcclusionMode>()
            .register_type::<FogOfWarConfig>()
            .register_type::<FogOfWarStats>()
            .register_type::<FogOverlay2d>()
            .register_type::<FogPalette>()
            .register_type::<FogProjectionReceiver>()
            .register_type::<FogRevealShape>()
            .register_type::<FogVisibilityState>()
            .register_type::<FogWorldAxes>()
            .register_type::<VisionCellSource>()
            .register_type::<VisionOccluder>()
            .register_type::<VisionSource>()
            .configure_sets(
                self.update_schedule,
                (
                    FogOfWarSystems::CollectVisionSources,
                    FogOfWarSystems::ComputeVisibility,
                    FogOfWarSystems::UpdateExplorationMemory,
                    FogOfWarSystems::UploadRenderData,
                )
                    .chain(),
            )
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .add_systems(
                self.update_schedule,
                systems::collect_inputs
                    .in_set(FogOfWarSystems::CollectVisionSources)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::compute_visibility
                    .in_set(FogOfWarSystems::ComputeVisibility)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::update_exploration_memory
                    .in_set(FogOfWarSystems::UpdateExplorationMemory)
                    .run_if(systems::runtime_is_active),
            );

        rendering::plugin(app, self.update_schedule);
    }
}
