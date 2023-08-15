use sauron::{html::*, *};
/// a utility enum which hold each cases of line selection
pub(super) enum SelectionSplits {
    /// the whole range/line is selected
    SelectAll(String),
    /// the first part is plain, the second one is selected
    SelectRight(String, String),
    /// the first part is plain, the second one is selected, the third one is plain
    SelectMiddle(String, String, String),
    /// the first part is selected, the second one is plain
    SelectLeft(String, String),
    /// It is not part of the selection
    NotSelected(String),
}

impl<MSG> Component<MSG, ()> for SelectionSplits
where
    MSG: 'static,
{
    fn update(&mut self, _msg: MSG) -> Effects<MSG, ()> {
        Effects::none()
    }
    fn view(&self) -> Node<MSG> {
        match self {
            Self::SelectAll(line) => span([Self::class_ns("selected")], [text(line)]),
            Self::SelectRight(first, second) => span(
                [],
                [
                    span([], [text(first)]),
                    span([Self::class_ns("selected")], [text(second)]),
                ],
            ),
            Self::SelectMiddle(first, second, third) => span(
                [],
                [
                    span([], [text(first)]),
                    span([Self::class_ns("selected")], [text(second)]),
                    span([], [text(third)]),
                ],
            ),
            Self::SelectLeft(first, second) => span(
                [],
                [
                    span([Self::class_ns("selected")], [text(first)]),
                    span([], [text(second)]),
                ],
            ),
            Self::NotSelected(line) => span([], [text(line)]),
        }
    }
}

impl SelectionSplits {
    pub(super) fn view_with_style<MSG>(&self, node_style: Attribute<MSG>) -> Node<MSG>
    where
        MSG: 'static,
    {
        match self {
            Self::SelectAll(line) => span([Self::class_ns("selected"), node_style], [text(line)]),
            Self::SelectRight(first, second) => span(
                [node_style],
                [
                    span([], [text(first)]),
                    span([Self::class_ns("selected")], [text(second)]),
                ],
            ),
            Self::SelectMiddle(first, second, third) => span(
                [node_style],
                [
                    span([], [text(first)]),
                    span([Self::class_ns("selected")], [text(second)]),
                    span([], [text(third)]),
                ],
            ),
            Self::SelectLeft(first, second) => span(
                [node_style],
                [
                    span([Self::class_ns("selected")], [text(first)]),
                    span([], [text(second)]),
                ],
            ),
            Self::NotSelected(line) => span([node_style], [text(line)]),
        }
    }
}
