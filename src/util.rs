use css_colors::RGBA;
use syntect::highlighting::Color;

pub(crate) fn to_rgba(color: Color) -> RGBA {
    let Color { r, g, b, a } = color;
    css_colors::rgba(r, g, b, a as f32 / 255.0)
}
