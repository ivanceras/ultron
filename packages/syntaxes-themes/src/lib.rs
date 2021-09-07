#![deny(warnings)]
use once_cell::sync::Lazy;
use syntect::dumps;
pub use syntect::easy::HighlightLines;
pub use syntect::highlighting::{Color, Style, Theme, ThemeSet};
pub use syntect::parsing::{SyntaxReference, SyntaxSet};

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!("../dump/syntaxes.packdump"))
});

pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!("../dump/themes.themedump"))
});
