use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::highlighting::Theme;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;

pub struct TextHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl Default for TextHighlighter {
    fn default() -> Self {
        let syntax_set: SyntaxSet = SyntaxSet::load_defaults_newlines();
        let theme_set: ThemeSet = ThemeSet::load_defaults();
        //let theme_name = "Solarized (dark)".to_string();
        let theme_name = "Solarized (light)".to_string();
        //let theme_name = "base16-eighties.dark".to_string();
        //let theme_name = "base16-ocean.dark".to_string();
        //let theme_name = "base16-mocha.dark".to_string();
        //let theme_name = "base16-ocean.light".to_string();
        let _active_theme = &theme_set.themes[&theme_name];

        Self {
            syntax_set,
            theme_set,
            theme_name,
        }
    }
}

impl TextHighlighter {
    pub(super) fn get_line_highlighter(&self) -> (HighlightLines, &SyntaxSet) {
        let syntax: &SyntaxReference = self
            .syntax_set
            .find_syntax_by_extension("rs")
            .expect("unable to find rust syntax reference");
        (
            HighlightLines::new(syntax, self.active_theme()),
            &self.syntax_set,
        )
    }

    pub(super) fn active_theme(&self) -> &Theme {
        &self.theme_set.themes[&self.theme_name]
    }
}
