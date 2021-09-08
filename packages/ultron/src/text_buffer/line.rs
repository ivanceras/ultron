#[cfg(feature = "with-dom")]
use super::Cell;
use super::Range;
use crate::TextBuffer;
use crate::COMPONENT_NAME;
use css_colors::Color;
use sauron::html::attributes;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;
#[allow(unused)]
use ultron_syntaxes_themes::Style;

#[derive(Debug)]
pub(super) struct Line {
    pub(super) ranges: Vec<Range>,
    /// total width of this line
    pub(super) width: usize,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            ranges: vec![Range::default()],
            width: 0,
        }
    }
}

impl Line {
    pub(super) fn from_ranges(ranges: Vec<Range>) -> Self {
        Self {
            width: ranges.iter().map(|range| range.width).sum(),
            ranges,
        }
    }

    /// get the text content of this line
    pub(super) fn text(&self) -> String {
        String::from_iter(
            self.ranges
                .iter()
                .flat_map(|range| range.cells.iter().map(|cell| cell.ch)),
        )
    }

    /// calcultate which column position for this x relative to the widths
    pub(super) fn calc_range_cell_index_position(
        &self,
        x: usize,
    ) -> Option<(usize, usize)> {
        let mut col_width = 0;
        for (i, range) in self.ranges.iter().enumerate() {
            for (j, cell) in range.cells.iter().enumerate() {
                if col_width >= x {
                    return Some((i, j));
                }
                col_width += cell.width;
            }
        }
        None
    }

    #[allow(unused)]
    pub(super) fn last_range_cell_index(&self) -> (usize, usize) {
        let line_ranges_len = self.ranges.len();
        let last = if line_ranges_len > 0 {
            line_ranges_len - 1
        } else {
            0
        };

        (
            last,
            self.ranges
                .last()
                .map(|ranges| ranges.cells.len())
                .unwrap_or(0),
        )
    }

    pub(super) fn view_line<MSG>(
        &self,
        text_buffer: &TextBuffer,
        line_index: usize,
    ) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let classes_ns_flag = |class_name_flags| {
            classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };
        let is_focused = text_buffer.is_focused_line(line_index);
        div(
            vec![
                key(line_index),
                class_ns("number__line"),
                classes_ns_flag([("line_focused", is_focused)]),
            ],
            vec![
                view_if(
                    text_buffer.options.show_line_numbers,
                    div(
                        vec![
                            class_ns("number"),
                            if let Some(gutter_bg) =
                                text_buffer.gutter_background()
                            {
                                style! {
                                    background_color: gutter_bg.to_css(),
                                }
                            } else {
                                empty_attr()
                            },
                            if let Some(gutter_fg) =
                                text_buffer.gutter_foreground()
                            {
                                style! {
                                    color: gutter_fg.to_css(),
                                }
                            } else {
                                empty_attr()
                            },
                        ],
                        vec![text(line_index + 1)],
                    ),
                ),
                div(
                    vec![class_ns("line")],
                    self.ranges.iter().enumerate().map(
                        |(range_index, range)| {
                            range.view_range(
                                text_buffer,
                                line_index,
                                range_index,
                            )
                        },
                    ),
                ),
            ],
        )
    }
}

#[cfg(feature = "with-dom")]
impl Line {
    /// append to the last range if there is none create a new range
    pub(super) fn push_char(&mut self, ch: char) {
        let cell = Cell::from_char(ch);
        self.push_cell(cell);
    }

    pub(super) fn push_cell(&mut self, cell: Cell) {
        if let Some(last_range) = self.ranges.last_mut() {
            self.width += cell.width;
            last_range.push_cell(cell);
        } else {
            let range = Range::from_cells(vec![cell], Style::default());
            self.push_range(range);
        }
    }

    pub(super) fn push_range(&mut self, range: Range) {
        self.width += range.width;
        self.ranges.push(range);
    }

    pub(super) fn replace_char(
        &mut self,
        range_index: usize,
        cell_index: usize,
        ch: char,
    ) {
        if let Some(range) = self.ranges.get_mut(range_index) {
            self.width -= range.width;
            let cell = Cell::from_char(ch);
            range.replace_cell(cell_index, cell);
            self.width += range.width;
        }
    }

    pub(super) fn insert_char(
        &mut self,
        range_index: usize,
        cell_index: usize,
        ch: char,
    ) {
        if let Some(range) = self.ranges.get_mut(range_index) {
            self.width -= range.width;
            let cell = Cell::from_char(ch);
            range.insert_cell(cell_index, cell);
            self.width += range.width;
        }
    }
}
