use crate::TextBuffer;
use crate::COMPONENT_NAME;
use sauron::html::attributes;
use sauron::prelude::*;
use sauron::Node;
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Copy, Debug)]
pub(super) struct Cell {
    pub(super) ch: char,
    /// width of this character
    pub(super) width: usize,
}

impl Cell {
    pub(super) fn from_char(ch: char) -> Self {
        Self {
            width: ch.width().expect("must have a unicode width"),
            ch,
        }
    }

    pub(super) fn view_cell<MSG>(
        &self,
        text_buffer: &TextBuffer<MSG>,
        page_index: usize,
        line_index: usize,
        range_index: usize,
        cell_index: usize,
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
        let is_focused = text_buffer.is_focused_cell(
            page_index,
            line_index,
            range_index,
            cell_index,
        );

        let is_selected = text_buffer.in_selection(
            page_index,
            line_index,
            range_index,
            cell_index,
        );

        span(
            [
                class_ns("ch"),
                classes_ns_flag([
                    ("ch_focused", is_focused),
                    ("selected", is_selected),
                    (&format!("wide{}", self.width), self.width > 1),
                ]),
            ],
            if text_buffer.options.show_cursor && is_focused {
                [div([class_ns("cursor")], [self.view_ch(text_buffer)])]
            } else {
                [self.view_ch(text_buffer)]
            },
        )
    }

    fn view_ch<MSG>(&self, text_buffer: &TextBuffer<MSG>) -> Node<MSG> {
        // we use nbsp; when rendering for static site generator
        // while using the actual whitespace when used/typing in the editor
        if text_buffer.options.use_for_ssg && self.ch.is_whitespace() {
            safe_html("&nbsp;")
        } else {
            text(self.ch)
        }
    }
}
