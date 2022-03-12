use css_colors::RGBA;
use nalgebra::Point2;
use std::fmt::Debug;
use ultron_syntaxes_themes::Color;

pub(crate) fn to_rgba(color: Color) -> RGBA {
    let Color { r, g, b, a } = color;
    css_colors::rgba(r, g, b, a as f32 / 255.0)
}

pub(crate) fn normalize_points<T>(
    p1: Point2<T>,
    p2: Point2<T>,
) -> (Point2<T>, Point2<T>)
where
    T: Copy + Clone + PartialEq + Debug + Ord + 'static,
{
    let min_x = p1.x.min(p2.x);
    let min_y = p1.y.min(p2.y);
    let max_x = p1.x.max(p2.x);
    let max_y = p1.y.max(p2.y);
    (Point2::new(min_x, min_y), Point2::new(max_x, max_y))
}

pub(crate) fn normalize_number(n1: usize, n2: usize) -> (usize, usize) {
    if n1 > n2 {
        (n2, n1)
    } else {
        (n1, n2)
    }
}
