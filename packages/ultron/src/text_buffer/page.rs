use super::Line;
use super::TextBuffer;
use crate::util;
use crate::CH_HEIGHT;
use itertools::Itertools;
use sauron::html::attributes::data;
use sauron::prelude::*;

#[derive(Debug)]
pub(super) struct Page {
    pub(super) lines: Vec<Line>,
    page_size: usize,
    /// pages are visible by default
    pub(super) visible: bool,
}

impl Page {
    pub fn with_page_size(page_size: usize) -> Self {
        Self {
            lines: vec![Line::default()],
            page_size,
            visible: true,
        }
    }
    /// fill up lines such that the total lines in this page will be equal to the page_size
    pub fn fill_page(&mut self) {
        let lines_len = self.lines.len();
        let line_gap = self.page_size.saturating_sub(lines_len);
        for _ in 0..line_gap {
            self.lines.push(Line::default());
        }
    }
    pub(super) fn from_lines(page_size: usize, lines: Vec<Line>) -> Vec<Self> {
        lines
            .into_iter()
            .chunks(page_size)
            .into_iter()
            .map(|chunks| Page {
                lines: chunks.collect(),
                page_size,
                visible: true,
            })
            .collect()
    }

    pub(super) fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub(super) fn delete_lines_to_end(&mut self, line_index: usize) {
        self.lines.drain(line_index..);
    }

    pub(super) fn get_text_lines_to_end(&self, line_index: usize) -> String {
        self.lines[line_index..]
            .iter()
            .map(|line| line.text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// delete lines from start_index to end_index (inclusive)
    pub(super) fn delete_lines(
        &mut self,
        start_index: usize,
        end_index: usize,
    ) {
        let (start_line, end_line) =
            util::normalize_number(start_index, end_index);
        self.lines.drain(start_index..=end_index);
    }

    pub(super) fn get_text_in_lines(
        &self,
        start_index: usize,
        end_index: usize,
    ) -> String {
        let (start_line, end_line) =
            util::normalize_number(start_index, end_index);
        self.lines[start_index..=end_index]
            .iter()
            .map(|line| line.text())
            .join("\n")
    }

    /// delete lines from start_line to end_line(exlcusive)
    pub(super) fn delete_lines_exclusive(
        &mut self,
        start_line: usize,
        end_line: usize,
    ) {
        let (start_line, end_line) =
            util::normalize_number(start_line, end_line);
        self.lines.drain(start_line..end_line);
    }

    pub(super) fn get_text_in_lines_exclusive(
        &self,
        start_line: usize,
        end_line: usize,
    ) -> String {
        let (start_line, end_line) =
            util::normalize_number(start_line, end_line);
        self.lines[start_line..end_line]
            .iter()
            .map(|line| line.text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) fn delete_cells_in_line(
        &mut self,
        line_index: usize,
        start_x: usize,
        end_x: usize,
    ) {
        self.lines[line_index].delete_cells(start_x, end_x);
    }

    pub(super) fn delete_cells_to_end(
        &mut self,
        line_index: usize,
        start_x: usize,
    ) {
        self.lines[line_index].delete_cells_to_end(start_x);
    }

    pub(super) fn get_text_to_end(
        &self,
        line_index: usize,
        start_x: usize,
    ) -> Option<String> {
        self.lines[line_index].get_text_to_end(start_x)
    }

    pub(super) fn get_text_in_line(
        &self,
        line_index: usize,
        start_x: usize,
        end_x: usize,
    ) -> Option<String> {
        self.lines[line_index].get_text(start_x, end_x)
    }

    pub(super) fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// the pixel height of this page
    pub(super) fn page_height(&self) -> u32 {
        self.total_lines() as u32 * CH_HEIGHT
    }

    pub(super) fn max_column(&self) -> usize {
        self.lines.iter().map(|line| line.width).max().unwrap_or(0)
    }
    pub(super) fn page_width(&self) -> usize {
        self.max_column()
    }

    /// the width of the line at line `n`
    pub(super) fn line_width(&self, line_index: usize) -> Option<usize> {
        self.lines.get(line_index).map(|l| l.width)
    }

    /// add more lines, used internally
    pub(super) fn add_lines(&mut self, n: usize) {
        for _i in 0..n {
            self.lines.push(Line::default());
        }
    }

    pub(super) fn ensure_line_exist(&mut self, line_index: usize) {
        let line_gap = line_index
            .saturating_add(1)
            .saturating_sub(self.total_lines());
        if line_gap > 0 {
            self.add_lines(line_gap);
        }
    }

    /// fill columns at line y putting a space in each of the cells
    pub(super) fn add_cell(&mut self, y: usize, n: usize) {
        let ch = ' ';
        for _i in 0..n {
            self.lines[y].push_char(ch);
        }
    }

    pub(super) fn join_line(&mut self, x: usize, y: usize) {
        if self.lines.get(y + 1).is_some() {
            let next_line = self.lines.remove(y + 1);
            self.lines[y].push_ranges(next_line.ranges);
        }
    }

    /// insert a character at this x and y and move cells after it to the right
    pub(super) fn insert_char_to_line(
        &mut self,
        line_index: usize,
        x: usize,
        ch: char,
    ) {
        let (range_index, cell_index) = self.lines[line_index]
            .calc_range_cell_index_position(x)
            .unwrap_or(self.lines[line_index].range_cell_next());

        self.lines[line_index].insert_char(range_index, cell_index, ch);
    }

    /// replace the character at this location
    pub(super) fn replace_char_to_line(
        &mut self,
        line_index: usize,
        x: usize,
        ch: char,
    ) -> Option<char> {
        let (range_index, cell_index) = self.lines[line_index]
            .calc_range_cell_index_position(x)
            .expect("the range_index and cell_index must have existed at this point");
        self.lines[line_index].replace_char(range_index, cell_index, ch)
    }

    pub(super) fn insert_line(&mut self, line_index: usize, line: Line) {
        self.lines.insert(line_index, line);
    }

    //TODO: delegrate the deletion of the char to the line and range
    /// delete character at this position
    pub(super) fn delete_char_to_line(
        &mut self,
        line_index: usize,
        x: usize,
    ) -> Option<char> {
        if let Some(line) = self.lines.get_mut(line_index) {
            if let Some((range_index, col)) =
                line.calc_range_cell_index_position(x)
            {
                if let Some(range) = line.ranges.get_mut(range_index) {
                    if range.cells.get(col).is_some() {
                        let cell = range.cells.remove(col);
                        return Some(cell.ch);
                    }
                }
            }
        }
        None
    }

    pub(super) fn view_page<MSG>(
        &self,
        text_buffer: &TextBuffer<MSG>,
        page_index: usize,
    ) -> Node<MSG> {
        div(
            [
                // skip diffing when this page is not shown
                //skip(!self.visible),
                class("page"),
                class(format!("page_{}", page_index)),
                style! {height: px(self.page_height())},
            ],
            if self.visible {
                self.lines
                    .iter()
                    .enumerate()
                    .map(|(line_index, line)| {
                        line.view_line(text_buffer, page_index, line_index)
                    })
                    .collect()
            } else {
                vec![comment("hidden for optimization")]
            },
        )
    }
}

impl ToString for Page {
    fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.text())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
