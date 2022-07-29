use crate::HighlightLines;
use crate::Style;
use crate::{Color, Theme, ThemeSet};
use crate::{SyntaxReference, SyntaxSet};

const DEFAULT_THEME: &str = "solarized-light";

pub struct TextHighlighter {
    syntax_set: &'static SyntaxSet,
    theme_set: &'static ThemeSet,
    theme_name: Option<String>,
    syntax_ref: Option<&'static SyntaxReference>,
    highlight_lines: Option<HighlightLines<'static>>,
}

impl Default for TextHighlighter {
    fn default() -> Self {
        let syntax_set: &SyntaxSet = &crate::SYNTAX_SET;
        let theme_set: &ThemeSet = &crate::THEME_SET;
        Self {
            syntax_set,
            theme_set,
            theme_name: None,
            syntax_ref: None,
            highlight_lines: None,
        }
    }
}

impl TextHighlighter {
    pub fn with_theme(theme_name: &str) -> Self {
        let mut text_highlighter = Self::default();
        text_highlighter.select_theme(theme_name);
        text_highlighter
    }

    pub fn set_syntax_token(&mut self, syntax_token: &str) {
        let syntax_ref: &SyntaxReference = self
            .syntax_set
            .find_syntax_by_token(syntax_token)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        self.syntax_ref = Some(syntax_ref);

        self.set_highlight_lines();
    }

    /// createa new HighlightLines and override existing
    /// to reset the underlying parse state
    fn set_highlight_lines(&mut self) {
        let syntax_ref = self
            .syntax_ref
            .as_ref()
            .expect("must have syntax_ref already");
        self.highlight_lines =
            Some(HighlightLines::new(syntax_ref, self.active_theme()));
    }

    /// reset syntect parse state by resetting highlight lines
    pub fn reset(&mut self) {
        self.set_highlight_lines();
    }

    /// set the theme name
    pub fn select_theme(&mut self, theme_name: &str) {
        if let Some(_) = self.theme_set.themes.get(theme_name) {
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

    pub fn highlight_line<'l>(
        &mut self,
        line: &'l str,
    ) -> Result<Vec<(Style, &'l str)>, syntect::Error> {
        let highlight_lines = self
            .highlight_lines
            .as_mut()
            .expect("must have a highlight line");
        highlight_lines.highlight_line(line, &self.syntax_set)
    }

    pub fn active_theme(&self) -> &'static Theme {
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
