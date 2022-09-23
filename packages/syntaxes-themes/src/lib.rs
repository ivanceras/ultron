#![deny(warnings)]
use once_cell::sync::Lazy;
use syntect::dumps;
pub use syntect::{
    easy::HighlightLines,
    highlighting::{
        Color,
        Style,
        Theme,
        ThemeSet,
    },
    parsing::{
        SyntaxReference,
        SyntaxSet,
    },
};
pub use text_highlighter::TextHighlighter;

mod text_highlighter;

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!("../dump/syntaxes.packdump"))
});

pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!("../dump/themes.themedump"))
});
