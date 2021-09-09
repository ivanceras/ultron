use crate::HighlightLines;
use crate::{Color, Theme, ThemeSet};
use crate::{SyntaxReference, SyntaxSet};

const DEFAULT_THEME: &str = "solarized-light";

pub struct TextHighlighter {
    syntax_set: &'static SyntaxSet,
    theme_set: &'static ThemeSet,
    theme_name: Option<String>,
}

impl Default for TextHighlighter {
    fn default() -> Self {
        let syntax_set: &SyntaxSet = &crate::SYNTAX_SET;
        let theme_set: &ThemeSet = &crate::THEME_SET;
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
            log::trace!("Setting theme to: {}", theme_name);
            self.theme_name = Some(theme_name.to_string());
        } else {
            format!("The valid theme names are: {:?}", self.get_theme_names());
            log::trace!(
                "The valid theme names are: {:?}",
                self.get_theme_names()
            );
            panic!("theme name: {} doesn't match", theme_name);
        }
    }

    fn get_theme_names(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }

    pub fn get_line_highlighter(
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

    pub fn active_theme(&self) -> &Theme {
        if let Some(theme_name) = self.theme_name.as_ref() {
            &self.theme_set.themes[theme_name]
        } else {
            &self.theme_set.themes[DEFAULT_THEME]
        }
    }

    pub fn gutter_background(&self) -> Option<Color> {
        self.active_theme().settings.gutter
    }

    pub fn gutter_foreground(&self) -> Option<Color> {
        self.active_theme().settings.gutter_foreground
    }

    pub fn theme_background(&self) -> Option<Color> {
        self.active_theme().settings.background
    }

    pub fn theme_foreground(&self) -> Option<Color> {
        self.active_theme().settings.foreground
    }

    pub fn selection_background(&self) -> Option<Color> {
        self.active_theme().settings.selection
    }
}
