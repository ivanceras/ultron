use ultron_core::{Editor, Options, Point2};
#[test]
fn test_text_selection() {
    let raw = "Hello world";
    let mut editor: Editor<()> = Editor::from_str(Options::default(), raw);
    editor.set_selection(Point2::new(0, 0), Point2::new(4, 0));
    let txt = editor.selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
}

#[test]
fn test_text_selection_world() {
    let raw = "Hello world";
    let mut editor: Editor<()> = Editor::from_str(Options::default(), raw);
    editor.set_selection(Point2::new(6, 0), Point2::new(10, 0));
    let txt = editor.selected_text();
    assert_eq!(txt, Some("world".to_string()));
}

#[test]
fn test_cut_text() {
    let raw = "Hello world";
    let mut editor = Editor::<()>::from_str(Options::default(), raw);
    editor.set_selection(Point2::new(0, 0), Point2::new(4, 0));
    let txt = editor.cut_selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
    assert_eq!(editor.get_content(), " world");
}

#[test]
fn test_cut_text_multi_line() {
    let raw = "before text\nHello world\nafter text";
    let mut editor = Editor::<()>::from_str(Options::default(), raw);
    editor.set_selection(Point2::new(0, 1), Point2::new(4, 1));
    let txt = editor.cut_selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
    assert_eq!(editor.get_content(), "before text\n world\nafter text");
}
