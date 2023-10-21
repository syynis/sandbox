use bevy::math::*;

pub fn box_lines(origin: Vec2, size: Vec2) -> [(Vec2, Vec2); 4] {
    let extend = size;
    let min = origin - Vec2::new(8., 8.);
    let max = origin + extend - Vec2::new(8., 8.);

    let bottom_right = (min, min + Vec2::new(size.x, 0.));
    let bottom_up = (min, min + Vec2::new(0., size.y));
    let top_left = (max, max - Vec2::new(size.x, 0.));
    let top_down = (max, max - Vec2::new(0., size.y));

    [bottom_right, bottom_up, top_left, top_down]
}
