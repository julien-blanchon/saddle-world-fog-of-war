use super::*;

#[test]
fn world_to_cell_and_center_round_trip() {
    let spec = FogGridSpec {
        origin: Vec2::new(10.0, 20.0),
        dimensions: UVec2::new(8, 6),
        cell_size: Vec2::new(2.0, 3.0),
        chunk_size: UVec2::new(4, 2),
    };

    let cell = spec
        .world_to_cell(Vec2::new(14.1, 26.9))
        .expect("position should map to a cell");
    assert_eq!(cell, IVec2::new(2, 2));
    assert_eq!(spec.cell_to_world_center(cell), Some(Vec2::new(15.0, 27.5)));
    assert_eq!(spec.index(cell), Some(18));
    assert_eq!(spec.cell_from_index(18), cell);
}

#[test]
fn chunk_addressing_stays_stable() {
    let spec = FogGridSpec {
        origin: Vec2::ZERO,
        dimensions: UVec2::new(10, 9),
        cell_size: Vec2::ONE,
        chunk_size: UVec2::new(4, 3),
    };

    assert_eq!(
        spec.chunk_for_cell(IVec2::new(0, 0)),
        Some(FogChunkCoord(UVec2::new(0, 0)))
    );
    assert_eq!(
        spec.chunk_for_cell(IVec2::new(7, 5)),
        Some(FogChunkCoord(UVec2::new(1, 1)))
    );
    assert_eq!(spec.chunk_count(), UVec2::new(3, 3));

    let bounds = spec.chunk_bounds(FogChunkCoord(UVec2::new(1, 1)));
    assert_eq!(bounds.min, IVec2::new(4, 3));
    assert_eq!(bounds.max, IVec2::new(8, 6));
}

#[test]
fn layer_mask_uses_64_slots() {
    assert_eq!(FogLayerMask::bit(FogLayerId::ZERO), FogLayerMask(1));
    assert_eq!(FogLayerMask::bit(FogLayerId(63)), FogLayerMask(1_u64 << 63));
}
