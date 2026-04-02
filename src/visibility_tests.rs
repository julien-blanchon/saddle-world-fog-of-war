use super::*;

use crate::{
    components::{FogOccluderShape, FogRevealShape},
    grid::{FogGridSpec, FogLayerId, FogLayerMask},
    resources::{FogOcclusionMode, FogOfWarConfig, FogOfWarMap},
};

fn test_map() -> FogOfWarMap {
    FogOfWarMap::new(FogOfWarConfig {
        grid: FogGridSpec {
            origin: Vec2::ZERO,
            dimensions: UVec2::new(12, 12),
            cell_size: Vec2::ONE,
            chunk_size: UVec2::splat(4),
        },
        occlusion_mode: FogOcclusionMode::Bresenham,
        ..default()
    })
}

#[test]
fn hidden_visible_explored_visible_flow_is_stable() {
    let mut map = test_map();
    let source = VisionSourceSample {
        layer: FogLayerId(0),
        position: Vec2::new(2.5, 2.5),
        shape: FogRevealShape::circle(2.2),
    };

    rebuild_blockers(&mut map, &[]);
    accumulate_visibility(&mut map, &[source]);
    commit_visibility(&mut map);
    assert_eq!(
        map.visibility_at_cell(FogLayerId(0), IVec2::new(2, 2)),
        Some(FogVisibilityState::Visible)
    );

    accumulate_visibility(&mut map, &[]);
    commit_visibility(&mut map);
    assert_eq!(
        map.visibility_at_cell(FogLayerId(0), IVec2::new(2, 2)),
        Some(FogVisibilityState::Explored)
    );

    accumulate_visibility(&mut map, &[source]);
    commit_visibility(&mut map);
    assert_eq!(
        map.visibility_at_cell(FogLayerId(0), IVec2::new(2, 2)),
        Some(FogVisibilityState::Visible)
    );
}

#[test]
fn overlapping_sources_merge_without_leaking_layers() {
    let mut map = test_map();
    let primary = VisionSourceSample {
        layer: FogLayerId(0),
        position: Vec2::new(4.5, 4.5),
        shape: FogRevealShape::circle(2.8),
    };
    let secondary = VisionSourceSample {
        layer: FogLayerId(0),
        position: Vec2::new(6.5, 4.5),
        shape: FogRevealShape::circle(2.8),
    };
    let other_layer = VisionSourceSample {
        layer: FogLayerId(1),
        position: Vec2::new(10.5, 10.5),
        shape: FogRevealShape::circle(1.5),
    };

    accumulate_visibility(&mut map, &[primary, secondary, other_layer]);
    commit_visibility(&mut map);

    assert!(map.is_visible(FogLayerId(0), IVec2::new(5, 4)));
    assert!(!map.is_visible(FogLayerId(1), IVec2::new(5, 4)));
    assert!(map.is_visible(FogLayerId(1), IVec2::new(10, 10)));
}

#[test]
fn arc_reveal_only_hits_forward_cells() {
    let mut map = test_map();
    let source = VisionSourceSample {
        layer: FogLayerId(0),
        position: Vec2::new(4.5, 4.5),
        shape: FogRevealShape::arc(4.0, std::f32::consts::FRAC_PI_2, Vec2::X),
    };

    accumulate_visibility(&mut map, &[source]);
    commit_visibility(&mut map);

    assert!(map.is_visible(FogLayerId(0), IVec2::new(7, 4)));
    assert!(!map.is_visible(FogLayerId(0), IVec2::new(2, 4)));
}

#[test]
fn blockers_stop_visibility_behind_walls() {
    let mut map = test_map();
    let occluder = VisionOccluderSample {
        layers: FogLayerMask::ALL,
        position: Vec2::new(6.5, 4.5),
        shape: FogOccluderShape::rect(Vec2::new(0.5, 2.5)),
    };
    let source = VisionSourceSample {
        layer: FogLayerId(0),
        position: Vec2::new(3.5, 4.5),
        shape: FogRevealShape::circle(6.0),
    };

    rebuild_blockers(&mut map, &[occluder]);
    accumulate_visibility(&mut map, &[source]);
    commit_visibility(&mut map);

    assert!(map.is_visible(FogLayerId(0), IVec2::new(5, 4)));
    assert!(!map.is_visible(FogLayerId(0), IVec2::new(8, 4)));
}
