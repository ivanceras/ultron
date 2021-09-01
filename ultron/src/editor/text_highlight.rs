use crate::editor::COMPONENT_NAME;
use sauron::html::attributes;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;

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

/// text manipulation
impl TextHighlight {
    /// the total number of lines of this text canvas
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// the width of the line at line `n`
    pub fn line_width(&self, n: usize) -> Option<usize> {
        self.lines.get(n).map(|l| l.width)
    }

    /// calcultate which column position for this x relative to the widths
    fn calc_range_col_insert_position(line: &Line, x: usize) -> (usize, usize) {
        println!("calculating range col where x is: {}", x);
        let mut col_width = 0;
        let mut index = 0;
        for (i, range) in line.ranges.iter().enumerate() {
            for (j, cell) in range.cells.iter().enumerate() {
                if col_width >= x {
                    return (i, j);
                }
                col_width += cell.width;
                index += 1;
            }
        }
        dbg!(&x);
        dbg!(&line);

        let line_ranges_len = line.ranges.len();
        let last = if line_ranges_len > 0 {
            line_ranges_len - 1
        } else {
            0
        };

        (
            last,
            line.ranges
                .last()
                .map(|ranges| ranges.cells.len())
                .unwrap_or(0),
        )
    }

    /// add more lines, used internally
    fn add_lines(&mut self, n: usize) {
        for _i in 0..n {
            println!("Adding line...{}", _i);
            self.lines.push(Line::default());
        }
        dbg!(&self.lines);
    }

    /// fill columns at line y putting a space in each of the cells
    fn add_col(&mut self, y: usize, n: usize) {
        let ch = ' ';
        for _i in 0..n {
            println!("adding to column {}: {:?}", y, ch);
            self.lines[y].push_char(ch);
        }
    }

    /// break at line y and put the characters after x on the next line
    pub fn break_line(&mut self, x: usize, y: usize) {
        if let Some(line) = self.lines.get_mut(y) {
            let (range_index, col) = Self::calc_range_col_insert_position(line, x);
            if let Some(range_bound) = line.ranges.get_mut(range_index) {
                let other = range_bound.split_at(col);
                let mut rest = line.ranges.drain(range_index + 1..).collect::<Vec<_>>();
                rest.insert(0, other);
                self.lines.insert(y + 1, Line::from_ranges(rest));
            }
        }
    }

    /// insert a character at this x and y and move cells after it to the right
    pub fn insert_char(&mut self, x: usize, y: usize, ch: char) {
        self.add_char(false, x, y, ch);
    }

    /// replace the character at this location
    pub fn replace_char(&mut self, x: usize, y: usize, ch: char) {
        self.add_char(true, x, y, ch);
    }

    /// delete character at this position
    pub fn delete_char(&mut self, x: usize, y: usize) {
        dbg!(&self.lines);
        if let Some(line) = self.lines.get_mut(y) {
            let (range_index, col) = dbg!(Self::calc_range_col_insert_position(line, x));
            if let Some(mut range) = line.ranges.get_mut(range_index) {
                if range.cells.get(col).is_some() {
                    range.cells.remove(col);
                }
            }
        }
    }

    fn add_char(&mut self, is_replace: bool, x: usize, y: usize, ch: char) {
        assert!(
            ch != '\n',
            "line breaks should have been pre-processed before this point"
        );
        assert!(
            ch != '\t',
            "tabs should have been pre-processed before this point"
        );
        let line_gap = if y > self.total_lines() {
            y - self.total_lines()
        } else {
            0
        };
        dbg!(&line_gap);

        if self.total_lines() == 0 {
            self.add_lines(1);
        }
        if line_gap > 0 {
            self.add_lines(line_gap);
        }
        let line = &self.lines[y];
        let col_diff = if x > line.width { x - line.width } else { 0 };
        dbg!(&col_diff);
        if col_diff > 0 {
            self.add_col(y, col_diff);
        }

        let ch_width = ch.width().expect("must have a unicode width");
        let cell = Cell {
            ch,
            width: ch_width,
        };

        dbg!(&x);
        dbg!(&y);

        let (range_index, char_index) =
            dbg!(Self::calc_range_col_insert_position(&self.lines[y], x));

        dbg!(&self.lines);

        dbg!(&self.lines[y]);

        if is_replace {
            self.lines[y].ranges[range_index].cells[char_index] = cell
        } else {
            self.lines[y].ranges[range_index]
                .cells
                .insert(char_index, cell);
        }
    }

    fn rehighlight_line(&mut self, y: usize) {
        if let Some(mut line) = self.lines.get_mut(y) {
            line.rehighlight(&self.highlighter);
        }
    }
}

impl Line {
    fn push_char(&mut self, ch: char) {
        let cell = Cell::from_char(ch);
        let range = Range::from_cells(vec![cell], Style::default());
        self.ranges.push(range);
    }

    /// rehighlight this line
    fn rehighlight(&mut self, highlighter: &Highlighter) {
        let line_str = self.text();
        let style_range: Vec<(Style, &str)> = highlighter.highlight(&line_str);
        self.ranges = style_range
            .into_iter()
            .map(|(style, range_str)| {
                let cells = range_str.chars().map(Cell::from_char).collect();
                Range::from_cells(cells, style)
            })
            .collect();
    }

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
            ranges: vec![Range::default()],
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

    fn split_at(&mut self, cell_index: usize) -> Self {
        let other = self.cells.split_off(cell_index);
        Self::from_cells(other, self.style)
    }

    fn view<MSG>(&self) -> Node<MSG> {
        let class_ns = |class_names| attributes::class_namespaced(COMPONENT_NAME, class_names);
        let background = self.style.background;
        let foreground = self.style.foreground;
        div(
            vec![
                class_ns("range"),
                style! {
                    color: format!("rgba({},{},{},{})", foreground.r,foreground.g, foreground.b, (foreground.a as f32/ 255.0)),
                    background_color: format!("rgba({},{},{},{})", background.r,background.g, background.b, (background.a as f32/ 255.0)),
                },
            ],
            self.cells
                .iter()
                .map(|cell| cell.view())
                .collect::<Vec<_>>(),
        )
    }
}

impl Default for Range {
    fn default() -> Self {
        Self {
            cells: vec![],
            width: 0,
            style: Style::default(),
        }
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
        let _active_theme = &theme_set.themes[&theme_name];

        Self {
            syntax_set,
            theme_set,
            theme_name,
        }
    }
}
