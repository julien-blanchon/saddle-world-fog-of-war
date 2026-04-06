mod components;
mod grid;
mod math;
mod messages;
mod persistence;
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
pub use persistence::{
    FogCustomPersistence, FogPersistenceCell, FogPersistenceMode, FogPersistencePolicy,
};
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
    ApplyPersistence,
    UploadRenderData,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct FogOfWarPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub config: FogOfWarConfig,
    custom_persistence: Option<FogCustomPersistence>,
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
            custom_persistence: None,
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_config(mut self, config: FogOfWarConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_custom_persistence(mut self, policy: impl FogPersistencePolicy) -> Self {
        self.custom_persistence = Some(FogCustomPersistence::new(policy));
        self
    }
}

pub struct FogOfWarRenderingPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl FogOfWarRenderingPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Default for FogOfWarPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Default for FogOfWarRenderingPlugin {
    fn default() -> Self {
        Self::new(Update)
    }
}

impl Plugin for FogOfWarPlugin {
    fn build(&self, app: &mut App) {
        let has_preinserted_custom = app.world().contains_resource::<FogCustomPersistence>();
        let mut config = self.config.clone();
        if self.custom_persistence.is_some() || has_preinserted_custom {
            config.persistence_mode = FogPersistenceMode::Custom;
        } else if config.persistence_mode == FogPersistenceMode::Custom {
            panic!(
                "FogPersistenceMode::Custom requires FogOfWarPlugin::with_custom_persistence(...) or a pre-inserted FogCustomPersistence resource"
            );
        }

        if let Some(custom_persistence) = &self.custom_persistence {
            app.insert_resource(custom_persistence.clone());
        }

        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.insert_resource(config.clone())
            .insert_resource(FogOfWarMap::new(config))
            .init_resource::<FogOfWarStats>()
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
            .register_type::<FogPersistenceMode>()
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
                    FogOfWarSystems::ApplyPersistence,
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
                systems::apply_persistence
                    .in_set(FogOfWarSystems::ApplyPersistence)
                    .run_if(systems::runtime_is_active),
            );
    }
}

impl Plugin for FogOfWarRenderingPlugin {
    fn build(&self, app: &mut App) {
        rendering::plugin(app, self.update_schedule);
    }
}
