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

    pub(super) fn push_ranges(&mut self, ranges: Vec<Range>) {
        self.width += ranges.iter().map(|range| range.width).sum::<usize>();
        self.ranges.extend(ranges);
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

    /// calculate the x position from range_index and cell_index of this line
    pub(crate) fn calc_range_cell_index_to_x(
        &self,
        range_index: usize,
        cell_index: usize,
    ) -> usize {
        let mut width = 0;
        for range in self.ranges[0..range_index].iter() {
            width += range.width;
        }
        for cell in self.ranges[range_index].cells[0..cell_index].iter() {
            width += cell.width;
        }
        width
    }

    /// The location of the the next cell to be appended to the last range of this line
    pub(super) fn range_cell_next(&self) -> (usize, usize) {
        (
            self.ranges.len().saturating_sub(1),
            self.ranges
                .last()
                .map(|ranges| ranges.cells.len())
                .unwrap_or(0),
        )
    }

    /// Return the indexes of the last range and the last cell of the last range of the line
    pub(super) fn range_cell_tail(&self) -> (usize, usize) {
        (
            self.ranges.len().saturating_sub(1),
            self.ranges
                .last()
                .map(|ranges| ranges.cells.len().saturating_sub(1))
                .unwrap_or(0),
        )
    }

    pub(super) fn view_line<MSG>(
        &self,
        text_buffer: &TextBuffer,
        page_index: usize,
        line_index: usize,
    ) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let classes_ns_flag = |class_name_flags| {
            classes_flag_namespaced(COMPONENT_NAME, class_name_flags)
        };
        let is_focused = text_buffer.is_focused_line(line_index);
        let line_number =
            page_index * text_buffer.options.page_size + line_index + 1;
        div(
            [
                key(line_number),
                class_ns("number__line"),
                classes_ns_flag([("line_focused", is_focused)]),
            ],
            [
                view_if(
                    text_buffer.options.show_line_numbers,
                    span(
                        [
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
                        [text(line_number)],
                    ),
                ),
                span(
                    [class_ns("line")],
                    self.ranges.iter().enumerate().map(
                        |(range_index, range)| {
                            range.view_range(
                                text_buffer,
                                page_index,
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
    ) -> Option<char> {
        if let Some(range) = self.ranges.get_mut(range_index) {
            self.width -= range.width;
            let cell = Cell::from_char(ch);
            let old_ch = range.replace_cell(cell_index, cell);
            self.width += range.width;
            Some(old_ch)
        } else {
            panic!("There should be a range");
        }
    }

    pub(super) fn recalc_width(&mut self) {
        self.width = self.ranges.iter().map(|range| range.width).sum();
    }

    /// delete from x to last
    pub(super) fn delete_cells_to_end(&mut self, start_x: usize) {
        if let Some((range_index, cell_index)) =
            self.calc_range_cell_index_position(start_x)
        {
            log::trace!("delete cells to end, starting from: {}", start_x);
            log::trace!(
                "range_index: {}, cell_index: {}",
                range_index,
                cell_index
            );
            self.ranges[range_index].delete_cells_to_end(cell_index);
            // drain all ranges beyond `range_index`
            self.ranges.drain(range_index + 1..);
            self.recalc_width();
        }
    }

    /*
    /// delete from the 0 to end_x
    pub(super) fn delete_cells_from_start(&mut self, end_x: usize) {
        if let Some((range_index, cell_index)) =
            self.calc_range_cell_index_position(end_x)
        {
            log::trace!("deleting cells from start to {}", end_x);
            log::trace!(
                "range_index: {}, cell_index: {}",
                range_index,
                cell_index
            );
            self.ranges[range_index].delete_cells_from_start(cell_index);
            // delete ranges from 0 to before this end_x location
            self.ranges.drain(0..range_index);
            self.recalc_width();
        } else {
            log::trace!("no deletion happened...");
        }
    }
    */

    /// delete cells on this line from `start_x` to `end_x`
    pub(super) fn delete_cells(&mut self, start_x: usize, end_x: usize) {
        println!("deleting cells from {} to {}", start_x, end_x);
        let start = self.calc_range_cell_index_position(start_x);

        // if None, it is because it beyond the cell index of this line, so we use the last_range
        // cell
        let end = self
            .calc_range_cell_index_position(end_x)
            .unwrap_or(self.range_cell_tail());
        if let Some((start_range, start_cell)) = start {
            let (end_range, end_cell) = end;
            if start_range == end_range {
                self.ranges[start_range].delete_cells(start_cell, end_cell);
            } else {
                self.ranges[end_range].delete_cells(0, end_cell);
                self.ranges[start_range].delete_cells_to_end(start_cell);
            }
            self.ranges.drain(start_range..end_range);
            self.recalc_width();
        }
    }

    /// get text from the 0 to end_x
    pub(super) fn get_text_from_start(
        &mut self,
        end_x: usize,
    ) -> Option<String> {
        if let Some((range_index, cell_index)) =
            self.calc_range_cell_index_position(end_x)
        {
            let end_text =
                self.ranges[range_index].get_text_from_start(cell_index);
            // delete ranges from 0 to before this end_x location
            let start_text = self.ranges[0..range_index]
                .iter()
                .map(|range| range.to_string())
                .collect::<Vec<_>>()
                .join("");

            Some([start_text, end_text].join(""))
        } else {
            None
        }
    }

    /// get the string content at this location
    pub(super) fn get_text(
        &self,
        start_x: usize,
        end_x: usize,
    ) -> Option<String> {
        println!("getting text from cells from {} to {}", start_x, end_x);
        let start = self.calc_range_cell_index_position(start_x);

        // if None, it is because it beyond the cell index of this line, so we use the last_range
        // cell
        let end = self
            .calc_range_cell_index_position(end_x)
            .unwrap_or(self.range_cell_tail());
        if let Some((start_range, start_cell)) = start {
            let (end_range, end_cell) = end;
            if start_range == end_range {
                let mid_text =
                    self.ranges[start_range].get_text(start_cell, end_cell);
                Some(mid_text)
            } else {
                let from_start =
                    self.ranges[end_range].get_text_from_start(end_cell);
                let to_end =
                    self.ranges[start_range].get_text_to_end(start_cell);
                let mid_text = self.ranges[start_range..end_range]
                    .iter()
                    .map(|range| range.to_string())
                    .collect::<Vec<_>>()
                    .join("");

                Some([from_start, mid_text, to_end].join(""))
            }
        } else {
            None
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
