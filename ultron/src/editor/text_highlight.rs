use crate::editor::COMPONENT_NAME;
use sauron::html::attributes;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;
use std::marker::PhantomData;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::highlighting::Theme;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxReference;
use syntect::parsing::SyntaxSet;
use unicode_width::UnicodeWidthChar;

/// A text buffer where every insertion of character it will
/// recompute the highlighting of a line
pub struct TextHighlight {
    lines: Vec<Line>,
    highlighter: Highlighter,
}

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

#[derive(Debug)]
pub struct Line {
    ranges: Vec<Range>,
    /// total width of this line
    width: usize,
}

#[derive(Debug)]
pub struct Range {
    cells: Vec<Cell>,
    width: usize,
    style: Style,
}

#[derive(Debug)]
pub struct Cell {
    ch: char,
    /// width of this character
    width: usize,
}

impl TextHighlight {
    pub fn from_str(content: &str) -> Self {
        let highlighter = Highlighter::default();
        let lines = content
            .lines()
            .map(|line| {
                let line_str = String::from_iter(line.chars());
                let style_range: Vec<(Style, &str)> = highlighter.highlight(&line_str);

                let ranges: Vec<Range> = style_range
                    .into_iter()
                    .map(|(style, range_str)| {
                        let cells = range_str.chars().map(Cell::from_char).collect();
                        Range::from_cells(cells, style)
                    })
                    .collect();

                Line::from_ranges(ranges)
            })
            .collect();
        Self { lines, highlighter }
    }

    /// the total number of lines of this text canvas
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// the width of the line at line `n`
    pub fn line_width(&self, n: usize) -> Option<usize> {
        self.lines.get(n).map(|l| l.width)
    }

    pub fn active_theme(&self) -> &Theme {
        self.highlighter.active_theme()
    }

    pub fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        div(
            vec![class_ns("code")],
            self.lines
                .iter()
                .map(|line| line.view())
                .collect::<Vec<_>>(),
        )
    }
}

impl Line {
    /// get the text content of this line
    fn text(&self) -> String {
        String::from_iter(
            self.ranges
                .iter()
                .flat_map(|range| range.cells.iter().map(|cell| cell.ch)),
        )
    }

    fn from_ranges(ranges: Vec<Range>) -> Self {
        Self {
            width: ranges.iter().map(|range| range.width).sum(),
            ranges,
        }
    }

    fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        div(
            vec![class_ns("line")],
            self.ranges
                .iter()
                .map(|range| range.view())
                .collect::<Vec<_>>(),
        )
    }
}

impl Default for Line {
    fn default() -> Self {
        Self {
            ranges: vec![],
            width: 0,
        }
    }
}

impl Range {
    fn from_cells(cells: Vec<Cell>, style: Style) -> Self {
        Self {
            width: cells.iter().map(|cell| cell.width).sum(),
            cells,
            style,
        }
    }

    fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        div(
            vec![class_ns("range")],
            self.cells
                .iter()
                .map(|cell| cell.view())
                .collect::<Vec<_>>(),
        )
    }
}

impl ToString for TextHighlight {
    fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Cell {
    fn from_char(ch: char) -> Self {
        Self {
            width: ch.width().expect("must have a unicode width"),
            ch,
        }
    }

    fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        div(vec![class_ns("ch")], vec![text(self.ch)])
    }
}

impl Highlighter {
    fn highlight<'b>(&self, line: &'b str) -> Vec<(Style, &'b str)> {
        let syntax: &SyntaxReference = self
            .syntax_set
            .find_syntax_by_extension("rs")
            .expect("unable to find rust syntax reference");
        let mut highlight_line = HighlightLines::new(syntax, self.active_theme());
        highlight_line.highlight(line, &self.syntax_set)
    }
    fn active_theme(&self) -> &Theme {
        &self.theme_set.themes[&self.theme_name]
    }
}

impl Highlighter {
    fn default() -> Self {
        let syntax_set: SyntaxSet = SyntaxSet::load_defaults_newlines();
        let theme_set: ThemeSet = ThemeSet::load_defaults();
        let theme_name = "base16-eighties.dark".to_string();
        let active_theme = &theme_set.themes[&theme_name];

        Self {
            syntax_set,
            theme_set,
            theme_name,
        }
    }
}
