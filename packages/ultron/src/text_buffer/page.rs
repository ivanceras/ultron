use super::Line;
use super::TextBuffer;
use crate::CH_HEIGHT;
use itertools::Itertools;
use sauron::html::attributes::data;
use sauron::prelude::*;

#[derive(Default)]
pub(super) struct Page {
    pub(super) lines: Vec<Line>,
    page_size: usize,
    visible: bool,
}

impl Page {
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

    pub(super) fn delete_lines_from_start(&mut self, line_index: usize) {
        self.lines.drain(0..=line_index);
    }

    /// delete lines from start_line to end_line(exlcusive)
    pub(super) fn delete_lines_exclusive(
        &mut self,
        start_line: usize,
        end_line: usize,
    ) {
        self.lines.drain(start_line..end_line);
    }
    pub(super) fn delete_cells(
        &mut self,
        line_index: usize,
        start_x: usize,
        end_x: usize,
    ) {
        self.lines[line_index].delete_cells(start_x, end_x);
    }

    pub(super) fn delete_cells_from_start(
        &mut self,
        line_index: usize,
        end_x: usize,
    ) {
        self.lines[line_index].delete_cells_from_start(end_x);
    }
    pub(super) fn delete_cells_to_end(
        &mut self,
        line_index: usize,
        start_x: usize,
    ) {
        self.lines[line_index].delete_cells_to_end(start_x);
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
        text_buffer: &TextBuffer,
        page_index: usize,
    ) -> Node<MSG> {
        div(
            [
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
