use ultron_syntaxes_themes::HighlightLines;
use ultron_syntaxes_themes::{SyntaxReference, SyntaxSet};
use ultron_syntaxes_themes::{Theme, ThemeSet};

const DEFAULT_THEME: &str = "solarized-light";

pub struct TextHighlighter {
    syntax_set: &'static SyntaxSet,
    theme_set: &'static ThemeSet,
    theme_name: Option<String>,
}

impl Default for TextHighlighter {
    fn default() -> Self {
        let syntax_set: &SyntaxSet = &ultron_syntaxes_themes::SYNTAX_SET;
        let theme_set: &ThemeSet = &ultron_syntaxes_themes::THEME_SET;
        Self {
            syntax_set,
            theme_set,
            theme_name: None,
        }
    }
}

impl TextHighlighter {
    /// set the theme name
    pub fn select_theme(&mut self, theme_name: &str) {
        if let Some(_) = self.theme_set.themes.get(theme_name) {
            self.theme_name = Some(theme_name.to_string());
        } else {
            format!("The valid theme names are: {:?}", self.get_theme_names());
            log::warn!("theme name: {} doesn't match", theme_name);
        }
    }

    fn get_theme_names(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }

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
        if let Some(theme_name) = self.theme_name.as_ref() {
            &self.theme_set.themes[theme_name]
        } else {
            &self.theme_set.themes[DEFAULT_THEME]
        }
    }
}
