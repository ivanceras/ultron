pub use crate::font_loader::FontSettings;
use ultron_core::BaseOptions;

#[derive(Clone, Debug)]
pub struct Options {
    pub base_options: BaseOptions,
    pub font_settings: Option<FontSettings>,
    /// allow the editor to show or hide pages for optimization
    /// Note: set this to false when using the editor as a headless buffer
    pub use_paging_optimization: bool,
    /// show line numbers
    pub show_line_numbers: bool,
    /// show the status line
    pub show_status_line: bool,
    /// show virtual cursor
    pub show_cursor: bool,
    /// apply background on the characters from syntax highlighter
    pub use_background: bool,
    /// The syntect theme name used for syntax highlighting
    pub theme_name: Option<String>,
    /// the syntax token used for text highlighting, usually the PL name ie: rust, typescript, sql
    pub syntax_token: String,
    /// whether or not the editor occupy the container element
    /// false means the editor only expands to the number of lines in the code
    pub occupy_container: bool,
    /// number of lines in a page, when paging up and down
    pub page_size: usize,
    /// a flag to use syntax highlighting or not
    pub use_syntax_highlighter: bool,
    /// a flag to do replace mode when there is no characters to the right
    /// and switch to insert mode when there is characters to the right
    pub use_smart_replace_insert: bool,
    /// allow the user to select text via browser native text selection
    pub allow_text_selection: bool,
    /// always put the cursor into view
    pub scroll_cursor_into_view: bool,
    /// show context menu when user right clicks on the editor
    pub enable_context_menu: bool,
    /// if enabled, the editor will listen to keypresses
    /// if disabled, it will not react to keypresses
    /// default: true
    pub enable_keypresses: bool,
    /// clicking on the editor will set the cursor location at the click location
    pub enable_click: bool,
    /// the calculated width of the character `0` in px
    /// this is affected by font sized and font used
    pub ch_width: Option<f32>,
    /// the calculated height of the character `0` in px
    pub ch_height: Option<f32>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            base_options: BaseOptions::default(),
            font_settings: None,
            use_paging_optimization: true,
            show_line_numbers: true,
            show_status_line: true,
            show_cursor: true,
            use_background: true,
            theme_name: None,
            syntax_token: "txt".to_string(),
            occupy_container: true,
            page_size: 20,
            use_syntax_highlighter: true,
            use_smart_replace_insert: false,
            allow_text_selection: true,
            scroll_cursor_into_view: false,
            enable_context_menu: false,
            enable_keypresses: true,
            enable_click: true,
            ch_width: None,
            ch_height: None,
        }
    }
}
