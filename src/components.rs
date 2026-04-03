use bevy::{color::LinearRgba, prelude::*};

use crate::grid::{FogLayerId, FogLayerMask};

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum FogRevealShape {
    Circle {
        radius: f32,
    },
    Arc {
        radius: f32,
        angle_radians: f32,
        facing: Vec2,
    },
    Rect {
        half_extents: Vec2,
    },
}

impl FogRevealShape {
    pub fn circle(radius: f32) -> Self {
        Self::Circle {
            radius: radius.max(0.0),
        }
    }

    pub fn arc(radius: f32, angle_radians: f32, facing: Vec2) -> Self {
        Self::Arc {
            radius: radius.max(0.0),
            angle_radians: angle_radians.max(0.0),
            facing,
        }
    }

    pub fn rect(half_extents: Vec2) -> Self {
        Self::Rect {
            half_extents: half_extents.max(Vec2::ZERO),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct VisionSource {
    pub layer: FogLayerId,
    pub shared_layers: FogLayerMask,
    pub shape: FogRevealShape,
    pub offset: Vec2,
    pub enabled: bool,
}

impl VisionSource {
    pub fn new(layer: FogLayerId, shape: FogRevealShape) -> Self {
        Self {
            layer,
            shared_layers: FogLayerMask::EMPTY,
            shape,
            offset: Vec2::ZERO,
            enabled: true,
        }
    }

    pub fn circle(layer: FogLayerId, radius: f32) -> Self {
        Self::new(layer, FogRevealShape::circle(radius))
    }

    pub fn arc(layer: FogLayerId, radius: f32, angle_radians: f32, facing: Vec2) -> Self {
        Self::new(layer, FogRevealShape::arc(radius, angle_radians, facing))
    }

    pub fn rect(layer: FogLayerId, half_extents: Vec2) -> Self {
        Self::new(layer, FogRevealShape::rect(half_extents))
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_shared_layers(mut self, shared_layers: FogLayerMask) -> Self {
        self.shared_layers = shared_layers;
        self
    }

    pub fn resolved_layers(self) -> FogLayerMask {
        FogLayerMask::bit(self.layer).union(self.shared_layers)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum FogOccluderShape {
    Cell,
    Circle { radius: f32 },
    Rect { half_extents: Vec2 },
}

impl FogOccluderShape {
    pub fn rect(half_extents: Vec2) -> Self {
        Self::Rect {
            half_extents: half_extents.max(Vec2::ZERO),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct VisionOccluder {
    pub layers: FogLayerMask,
    pub shape: FogOccluderShape,
    pub offset: Vec2,
    pub enabled: bool,
}

impl VisionOccluder {
    pub fn new(layers: FogLayerMask, shape: FogOccluderShape) -> Self {
        Self {
            layers,
            shape,
            offset: Vec2::ZERO,
            enabled: true,
        }
    }

    pub fn cell(layers: FogLayerMask) -> Self {
        Self::new(layers, FogOccluderShape::Cell)
    }

    pub fn rect(layers: FogLayerMask, half_extents: Vec2) -> Self {
        Self::new(layers, FogOccluderShape::rect(half_extents))
    }

    pub fn circle(layers: FogLayerMask, radius: f32) -> Self {
        Self::new(
            layers,
            FogOccluderShape::Circle {
                radius: radius.max(0.0),
            },
        )
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct VisionCellSource {
    pub layers: FogLayerMask,
    pub cells: Vec<IVec2>,
    pub enabled: bool,
}

impl VisionCellSource {
    pub fn new(layers: FogLayerMask) -> Self {
        Self {
            layers,
            cells: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_cells(mut self, cells: Vec<IVec2>) -> Self {
        self.cells = cells;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct FogPalette {
    pub hidden: LinearRgba,
    pub explored: LinearRgba,
    pub visible: LinearRgba,
}

impl FogPalette {
    pub fn grayscale() -> Self {
        Self {
            hidden: LinearRgba::new(0.02, 0.03, 0.05, 0.92),
            explored: LinearRgba::new(0.18, 0.24, 0.30, 0.72),
            visible: LinearRgba::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn cinematic() -> Self {
        Self {
            hidden: LinearRgba::new(0.03, 0.05, 0.08, 0.95),
            explored: LinearRgba::new(0.16, 0.21, 0.27, 0.70),
            visible: LinearRgba::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl Default for FogPalette {
    fn default() -> Self {
        Self::grayscale()
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct FogOverlay2d {
    pub layer: FogLayerId,
    pub world_origin: Vec2,
    pub world_size: Vec2,
    pub palette: FogPalette,
    pub opacity: f32,
    pub edge_softness: f32,
    pub z: f32,
}

impl FogOverlay2d {
    pub fn new(layer: FogLayerId, world_origin: Vec2, world_size: Vec2) -> Self {
        Self {
            layer,
            world_origin,
            world_size,
            palette: FogPalette::default(),
            opacity: 1.0,
            edge_softness: 0.2,
            z: 20.0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct FogProjectionReceiver {
    pub layer: FogLayerId,
    pub world_origin: Vec2,
    pub world_size: Vec2,
    pub palette: FogPalette,
    pub opacity: f32,
    pub edge_softness: f32,
    pub elevation: f32,
}

impl FogProjectionReceiver {
    pub fn new(layer: FogLayerId, world_origin: Vec2, world_size: Vec2) -> Self {
        Self {
            layer,
            world_origin,
            world_size,
            palette: FogPalette::default(),
            opacity: 1.0,
            edge_softness: 0.25,
            elevation: 0.03,
        }
    }
}
