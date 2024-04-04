use ultron_core::{BaseEditor, BaseOptions, Point2, SelectionMode};
#[test]
fn test_text_selection() {
    let raw = "Hello world";
    let mut editor: BaseEditor<()> = BaseEditor::from_str(&BaseOptions::default(), raw);
    editor.set_selection(Point2::new(0, 0), Point2::new(4, 0));
    let txt = editor.selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
}

#[test]
fn test_text_selection_world() {
    let raw = "Hello world";
    let mut editor: BaseEditor<()> = BaseEditor::from_str(&BaseOptions::default(), raw);
    editor.set_selection(Point2::new(6, 0), Point2::new(10, 0));
    let txt = editor.selected_text();
    assert_eq!(txt, Some("world".to_string()));
}

#[test]
fn test_cut_text() {
    let raw = "Hello world";
    let mut editor = BaseEditor::<()>::from_str(&BaseOptions::default(), raw);
    editor.set_selection(Point2::new(0, 0), Point2::new(4, 0));
    let txt = editor.cut_selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
    assert_eq!(editor.get_content(), " world");
}

#[test]
fn test_cut_text_multi_line() {
    let raw = "before text\nHello world\nafter text";
    let mut editor = BaseEditor::<()>::from_str(&BaseOptions::default(), raw);
    editor.set_selection(Point2::new(0, 1), Point2::new(4, 1));
    let txt = editor.cut_selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
    assert_eq!(editor.get_content(), "before text\n world\nafter text");
}

#[test]
fn test_text_is_selected_linear() {
    let raw = "Hello world";
    let mut editor: BaseEditor<()> = BaseEditor::from_str(&BaseOptions::default(), raw);
    editor.set_selection(Point2::new(6, 0), Point2::new(10, 0));
    assert!(editor.is_selected(Point2::new(6, 0)));
    assert!(editor.is_selected(Point2::new(7, 0)));
    assert!(editor.is_selected(Point2::new(8, 0)));
    assert!(editor.is_selected(Point2::new(9, 0)));
    assert!(editor.is_selected(Point2::new(10, 0)));

    assert!(!editor.is_selected(Point2::new(5, 0)));
    assert!(!editor.is_selected(Point2::new(7, 1)));
    assert!(!editor.is_selected(Point2::new(11, 0)));
}

#[test]
fn test_text_is_selected_linear_multiline() {
    let raw = "Hello\nworld\nand you with \neveryone";
    let mut editor: BaseEditor<()> = BaseEditor::from_str(&BaseOptions::default(), raw);
    editor.set_selection(Point2::new(1, 1), Point2::new(2, 2));
    let txt = editor.selected_text();
    assert_eq!(txt, Some("orld\nand".to_string()));
    assert!(editor.is_selected(Point2::new(1, 1)));
    assert!(editor.is_selected(Point2::new(2, 1)));
    assert!(editor.is_selected(Point2::new(3, 1)));
    assert!(editor.is_selected(Point2::new(4, 1)));
    assert!(editor.is_selected(Point2::new(5, 1)));
    assert!(editor.is_selected(Point2::new(0, 2)));
    assert!(editor.is_selected(Point2::new(1, 2)));
    assert!(editor.is_selected(Point2::new(2, 2)));

    assert!(!editor.is_selected(Point2::new(3, 2)));
    assert!(!editor.is_selected(Point2::new(0, 0)));
    assert!(!editor.is_selected(Point2::new(0, 1)));
    assert!(!editor.is_selected(Point2::new(2, 0)));
    assert!(!editor.is_selected(Point2::new(3, 0)));
    assert!(!editor.is_selected(Point2::new(4, 0)));
}

#[test]
fn test_text_is_selected_block_multiline() {
    let raw = "Hello\nworld\nand you with \neveryone";
    let mut editor: BaseEditor<()> = BaseEditor::from_str(
        &BaseOptions {
            selection_mode: SelectionMode::Block,
            ..Default::default()
        },
        raw,
    );
    editor.set_selection(Point2::new(1, 1), Point2::new(2, 2));
    let txt = editor.selected_text();
    assert_eq!(txt, Some("or\nnd".to_string()));
    assert!(editor.is_selected(Point2::new(1, 1)));
    assert!(editor.is_selected(Point2::new(2, 1)));
    assert!(editor.is_selected(Point2::new(1, 2)));
    assert!(editor.is_selected(Point2::new(2, 2)));

    assert!(!editor.is_selected(Point2::new(5, 1)));
    assert!(!editor.is_selected(Point2::new(0, 2)));

    assert!(!editor.is_selected(Point2::new(3, 2)));
    assert!(!editor.is_selected(Point2::new(0, 0)));
    assert!(!editor.is_selected(Point2::new(0, 1)));
    assert!(!editor.is_selected(Point2::new(2, 0)));
    assert!(!editor.is_selected(Point2::new(3, 0)));
    assert!(!editor.is_selected(Point2::new(4, 0)));
}
