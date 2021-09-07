#![deny(warnings)]
use once_cell::sync::Lazy;
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub use syntect;

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!("../dump/syntaxes.packdump"))
});

pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!("../dump/themes.themedump"))
});
