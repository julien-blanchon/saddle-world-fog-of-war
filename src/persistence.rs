use std::sync::Arc;

use bevy::prelude::*;

use crate::grid::{FogLayerId, FogVisibilityState};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum FogPersistenceMode {
    NoMemory,
    #[default]
    ExploredMemory,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FogPersistenceCell {
    pub layer: FogLayerId,
    pub cell: IVec2,
    pub visible_now: bool,
    pub previous_state: FogVisibilityState,
}

pub trait FogPersistencePolicy: Send + Sync + 'static {
    fn commit_cell(&self, cell: FogPersistenceCell) -> FogVisibilityState;

    fn deactivate_cell(&self, cell: FogPersistenceCell) -> FogVisibilityState {
        self.commit_cell(FogPersistenceCell {
            visible_now: false,
            ..cell
        })
    }
}

#[derive(Resource, Clone)]
pub struct FogCustomPersistence {
    policy: Arc<dyn FogPersistencePolicy>,
}

impl FogCustomPersistence {
    pub fn new(policy: impl FogPersistencePolicy) -> Self {
        Self {
            policy: Arc::new(policy),
        }
    }

    pub(crate) fn commit_cell(&self, cell: FogPersistenceCell) -> FogVisibilityState {
        self.policy.commit_cell(cell)
    }

    pub(crate) fn deactivate_cell(&self, cell: FogPersistenceCell) -> FogVisibilityState {
        self.policy.deactivate_cell(cell)
    }
}

pub(crate) fn commit_builtin_cell(
    mode: FogPersistenceMode,
    cell: FogPersistenceCell,
) -> FogVisibilityState {
    match mode {
        FogPersistenceMode::NoMemory => {
            if cell.visible_now {
                FogVisibilityState::Visible
            } else {
                FogVisibilityState::Hidden
            }
        }
        FogPersistenceMode::ExploredMemory => {
            if cell.visible_now {
                FogVisibilityState::Visible
            } else if cell.previous_state == FogVisibilityState::Hidden {
                FogVisibilityState::Hidden
            } else {
                FogVisibilityState::Explored
            }
        }
        FogPersistenceMode::Custom => {
            panic!(
                "FogPersistenceMode::Custom requires a FogCustomPersistence resource or FogOfWarPlugin::with_custom_persistence(...)"
            );
        }
    }
}
