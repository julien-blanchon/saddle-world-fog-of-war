use bevy::prelude::*;

pub(crate) fn safe_normalize_or(value: Vec2, fallback: Vec2) -> Vec2 {
    let normalized = value.try_normalize();
    normalized
        .or_else(|| fallback.try_normalize())
        .unwrap_or(Vec2::X)
}

pub(crate) fn bresenham_line(start: IVec2, end: IVec2) -> Vec<IVec2> {
    let mut points = Vec::new();

    let mut x0 = start.x;
    let mut y0 = start.y;
    let x1 = end.x;
    let y1 = end.y;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        points.push(IVec2::new(x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }

        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x0 += sx;
        }
        if doubled <= dx {
            error += dx;
            y0 += sy;
        }
    }

    points
}
