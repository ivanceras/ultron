use once_cell::sync::Lazy;
use syntect::dumps;
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;

pub(crate) static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!(
        "../../../syntaxes-themes/dump/syntaxes.packdump"
    ))
});

pub(crate) static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    dumps::from_binary(include_bytes!(
        "../../../syntaxes-themes/dump/themes.themedump"
    ))
});

pub struct TextHighlighter {
    syntax_set: &'static SyntaxSet,
    theme_set: &'static ThemeSet,
    theme_name: String,
}

impl Default for TextHighlighter {
    fn default() -> Self {
        let syntax_set: &SyntaxSet = &SYNTAX_SET;
        let theme_set: &ThemeSet = &THEME_SET;
        let theme_name = "solarized-light".to_string();
        //let theme_name = "gruvbox-dark".to_string();

        for (name, _) in theme_set.themes.iter() {
            log::trace!("name: {}", name);
        }
        Self {
            syntax_set,
            theme_set,
            theme_name,
        }
    }
}

impl TextHighlighter {
    pub(crate) fn get_line_highlighter(
        &self,
        syntax_token: &str,
    ) -> (HighlightLines, &SyntaxSet) {
        let syntax: &SyntaxReference = self
            .syntax_set
            .find_syntax_by_token(syntax_token)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        (
            HighlightLines::new(syntax, self.active_theme()),
            &self.syntax_set,
        )
    }

    pub(crate) fn active_theme(&self) -> &Theme {
        &self.theme_set.themes[&self.theme_name]
    }
}
