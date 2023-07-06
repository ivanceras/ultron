use crate::SelectionMode;

#[derive(Clone, Debug)]
pub struct BaseOptions {
    /// block selection uses rectangular text selection
    /// linear selection select lines as most text editors
    pub selection_mode: SelectionMode,
    /// allow the click outside of the bounds of the text content editor
    pub use_virtual_edit: bool,
}

impl Default for BaseOptions {
    fn default() -> Self {
        Self {
            selection_mode: SelectionMode::Linear,
            use_virtual_edit: false,
        }
    }
}
