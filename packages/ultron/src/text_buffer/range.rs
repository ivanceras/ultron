use super::Cell;
use crate::util;
use crate::TextBuffer;
use crate::COMPONENT_NAME;
use css_colors::Color;
use sauron::html::attributes;
use sauron::prelude::*;
use sauron::Node;
use std::iter::FromIterator;
use ultron_syntaxes_themes::Style;

#[derive(Debug)]
pub(super) struct Range {
    pub(super) cells: Vec<Cell>,
    pub(super) width: usize,
    pub(super) style: Style,
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

impl Range {
    pub(super) fn from_cells(cells: Vec<Cell>, style: Style) -> Self {
        Self {
            width: cells.iter().map(|cell| cell.width).sum(),
            cells,
            style,
        }
    }

    #[allow(unused)]
    pub(super) fn text(&self) -> String {
        String::from_iter(self.cells.iter().map(|cell| cell.ch))
    }

    pub(super) fn view_range<MSG>(
        &self,
        text_buffer: &TextBuffer,
        line_index: usize,
        range_index: usize,
    ) -> Node<MSG> {
        let class_ns = |class_names| {
            attributes::class_namespaced(COMPONENT_NAME, class_names)
        };
        let classes_ns_flag = |class_name_flags| {
            attributes::classes_flag_namespaced(
                COMPONENT_NAME,
                class_name_flags,
            )
        };
        let background = util::to_rgba(self.style.background);
        let foreground = util::to_rgba(self.style.foreground);
        let is_focused = text_buffer.is_focused_range(line_index, range_index);
        span(
            [
                class_ns("range"),
                classes_ns_flag([("range_focused", is_focused)]),
                if text_buffer.options.use_background {
                    style! {
                        color: foreground.to_css(),
                        background_color: background.to_css(),
                    }
                } else {
                    empty_attr()
                },
            ],
            self.cells.iter().enumerate().map(|(cell_index, cell)| {
                cell.view_cell(text_buffer, line_index, range_index, cell_index)
            }),
        )
    }
}

impl Range {
    pub(super) fn recalc_width(&mut self) {
        self.width = self.cells.iter().map(|cell| cell.width).sum();
    }

    pub(super) fn push_cell(&mut self, cell: Cell) {
        self.width += cell.width;
        self.cells.push(cell);
    }

    pub(super) fn replace_cell(&mut self, cell_index: usize, new_cell: Cell) {
        if let Some(cell) = self.cells.get_mut(cell_index) {
            self.width -= cell.width;
            self.width += new_cell.width;
            *cell = new_cell;
        } else {
            panic!("There should be a cell");
        }
    }

    pub(super) fn insert_cell(&mut self, cell_index: usize, new_cell: Cell) {
        self.width += new_cell.width;
        self.cells.insert(cell_index, new_cell);
    }

    pub(super) fn split_at(&mut self, cell_index: usize) -> Self {
        let other = self.cells.split_off(cell_index);
        Self::from_cells(other, self.style)
    }

    /// delete the cells from cell_index to end
    pub(super) fn delete_cells_to_end(&mut self, cell_index: usize) {
        self.cells.drain(cell_index..);
        self.recalc_width();
    }

    pub(super) fn delete_cells_from_start(&mut self, cell_index: usize) {
        self.cells.drain(0..=cell_index);
    }

    pub(super) fn delete_cells(
        &mut self,
        start_index: usize,
        end_index: usize,
    ) {
        self.cells.drain(start_index..=end_index);
    }
}
