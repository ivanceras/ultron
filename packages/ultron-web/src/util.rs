use crate::ultron_core::{Color, Point2};
use css_colors::RGBA;

pub fn to_rgba(color: Color) -> RGBA {
    let Color { r, g, b, a } = color;
    css_colors::rgba(r, g, b, a as f32 / 255.0)
}

pub fn clamp_to_edge(point: Point2<i32>) -> Point2<i32> {
    let x = if point.x < 0 { 0 } else { point.x };
    let y = if point.y < 0 { 0 } else { point.y };
    Point2::new(x, y)
}
