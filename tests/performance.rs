use bevy::prelude::*;
use saddle_world_fog_of_war::{
    FogLayerId, FogLayerMask, FogOfWarConfig, FogOfWarPlugin, FogOfWarStats, FogWorldAxes,
    VisionOccluder, VisionSource,
};

#[derive(Clone, Copy)]
struct SourceSpec {
    position: Vec2,
    source: VisionSource,
}

#[derive(Clone, Copy)]
struct OccluderSpec {
    position: Vec2,
    occluder: VisionOccluder,
}

#[test]
fn log_visibility_profiles() {
    let small = run_profile(
        "small_lab_like",
        FogOfWarConfig {
            grid: saddle_world_fog_of_war::FogGridSpec {
                origin: Vec2::ZERO,
                dimensions: UVec2::new(24, 18),
                cell_size: Vec2::splat(1.0),
                chunk_size: UVec2::splat(8),
            },
            world_axes: FogWorldAxes::XZ,
            ..default()
        },
        &[
            SourceSpec {
                position: Vec2::new(4.5, 4.5),
                source: VisionSource::circle(FogLayerId(0), 4.8),
            },
            SourceSpec {
                position: Vec2::new(20.5, 13.0),
                source: VisionSource::circle(FogLayerId(0), 4.0),
            },
            SourceSpec {
                position: Vec2::new(18.0, 6.5),
                source: VisionSource::arc(FogLayerId(1), 5.5, 1.2, Vec2::new(-1.0, 0.0)),
            },
        ],
        &[
            OccluderSpec {
                position: Vec2::new(11.5, 8.5),
                occluder: VisionOccluder::rect(FogLayerMask::ALL, Vec2::new(0.6, 5.0)),
            },
            OccluderSpec {
                position: Vec2::new(16.5, 5.0),
                occluder: VisionOccluder::rect(FogLayerMask::ALL, Vec2::new(3.5, 0.6)),
            },
        ],
    );

    let large = run_profile(
        "large_rts_like",
        FogOfWarConfig {
            grid: saddle_world_fog_of_war::FogGridSpec {
                origin: Vec2::ZERO,
                dimensions: UVec2::new(96, 64),
                cell_size: Vec2::splat(1.0),
                chunk_size: UVec2::splat(16),
            },
            world_axes: FogWorldAxes::XZ,
            ..default()
        },
        &make_large_sources(),
        &make_large_occluders(),
    );

    let blocker_heavy = run_profile(
        "blocker_heavy",
        FogOfWarConfig {
            grid: saddle_world_fog_of_war::FogGridSpec {
                origin: Vec2::ZERO,
                dimensions: UVec2::new(64, 64),
                cell_size: Vec2::splat(1.0),
                chunk_size: UVec2::splat(8),
            },
            world_axes: FogWorldAxes::XZ,
            ..default()
        },
        &[
            SourceSpec {
                position: Vec2::new(8.5, 8.5),
                source: VisionSource::circle(FogLayerId(0), 8.5),
            },
            SourceSpec {
                position: Vec2::new(52.5, 52.5),
                source: VisionSource::circle(FogLayerId(0), 8.5),
            },
        ],
        &make_blocker_grid(),
    );

    for (label, stats) in [
        ("small_lab_like", &small),
        ("large_rts_like", &large),
        ("blocker_heavy", &blocker_heavy),
    ] {
        println!(
            "{label}: compute={}us upload={}us sources={} occluders={} layers={} dirty_chunks={} visible={} explored={}",
            stats.last_compute_micros,
            stats.last_upload_micros,
            stats.source_count,
            stats.occluder_count,
            stats.layer_count,
            stats.dirty_chunk_count,
            stats.visible_cells_total,
            stats.explored_cells_total,
        );
    }

    assert!(small.visible_cells_total > 0);
    assert!(large.visible_cells_total > small.visible_cells_total);
    assert!(blocker_heavy.occluder_count > small.occluder_count);
}

fn run_profile(
    label: &str,
    config: FogOfWarConfig,
    sources: &[SourceSpec],
    occluders: &[OccluderSpec],
) -> FogOfWarStats {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(FogOfWarPlugin::default().with_config(config));

    for source in sources {
        app.world_mut().spawn((
            Name::new(format!("{label} source")),
            Transform::from_xyz(source.position.x, 0.0, source.position.y),
            GlobalTransform::from_xyz(source.position.x, 0.0, source.position.y),
            source.source,
        ));
    }

    for occluder in occluders {
        app.world_mut().spawn((
            Name::new(format!("{label} occluder")),
            Transform::from_xyz(occluder.position.x, 0.0, occluder.position.y),
            GlobalTransform::from_xyz(occluder.position.x, 0.0, occluder.position.y),
            occluder.occluder,
        ));
    }

    app.update();
    app.world().resource::<FogOfWarStats>().clone()
}

fn make_large_sources() -> Vec<SourceSpec> {
    let mut sources = Vec::new();
    for index in 0..12 {
        sources.push(SourceSpec {
            position: Vec2::new(10.0 + index as f32 * 4.5, 10.0 + (index % 6) as f32 * 6.0),
            source: VisionSource::circle(FogLayerId(0), 3.2),
        });
    }

    for position in [
        Vec2::new(22.0, 22.0),
        Vec2::new(54.0, 40.0),
        Vec2::new(76.0, 18.0),
    ] {
        sources.push(SourceSpec {
            position,
            source: VisionSource::circle(FogLayerId(0), 6.0),
        });
    }

    sources
}

fn make_large_occluders() -> Vec<OccluderSpec> {
    [18.0, 28.0, 50.0, 67.0, 81.0]
        .into_iter()
        .map(|x| OccluderSpec {
            position: Vec2::new(x, 30.0),
            occluder: VisionOccluder::rect(FogLayerMask::ALL, Vec2::new(1.5, 12.0)),
        })
        .collect()
}

fn make_blocker_grid() -> Vec<OccluderSpec> {
    let mut occluders = Vec::new();
    for x in (8..56).step_by(6) {
        occluders.push(OccluderSpec {
            position: Vec2::new(x as f32, 24.0),
            occluder: VisionOccluder::rect(FogLayerMask::ALL, Vec2::new(0.5, 14.0)),
        });
    }
    for y in (10..54).step_by(6) {
        occluders.push(OccluderSpec {
            position: Vec2::new(32.0, y as f32),
            occluder: VisionOccluder::rect(FogLayerMask::ALL, Vec2::new(14.0, 0.5)),
        });
    }
    occluders
}
