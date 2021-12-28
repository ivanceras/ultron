use crate::Context;
use crate::Options;
use crate::TextBuffer;
use nalgebra::Point2;

#[test]
fn line_length() {
    let raw = "Hello world";
    let buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    assert_eq!(buffer.total_lines(), 1);
    assert_eq!(buffer.line_width(0), Some(11));
}

#[test]
fn test_get_text() {
    let raw = "Hello world";
    let buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    let txt = buffer.get_text(Point2::new(0, 0), Point2::new(4, 0));
    assert_eq!(txt, "Hello");
}

#[test]
fn test_text_selection() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.set_selection(Point2::new(0, 0), Point2::new(4, 0));
    let txt = buffer.selected_text();
    assert_eq!(txt, Some("Hello".to_string()));
}

#[test]
fn test_get_text_world() {
    let raw = "Hello world";
    let buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    let txt = buffer.get_text(Point2::new(6, 0), Point2::new(10, 0));
    assert_eq!(txt, "world");
}

#[test]
fn test_text_selection_world() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.set_selection(Point2::new(6, 0), Point2::new(10, 0));
    let txt = buffer.selected_text();
    assert_eq!(txt, Some("world".to_string()));
}

#[test]
fn delete_last_char() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.delete_char(10, 0);
    assert_eq!(buffer.to_string(), "Hello worl");
}

#[test]
fn delete_first_char() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.delete_char(0, 0);
    assert_eq!(buffer.to_string(), "ello world");
}

#[test]
fn delete_5th_char() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.delete_char(5, 0);
    assert_eq!(buffer.to_string(), "Helloworld");
}

#[test]
fn delete_in_2nd_line() {
    let raw = "Hello\nworld\nthere";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.delete_char(0, 2);
    assert_eq!(buffer.to_string(), "Hello\nworld\nhere");
}

#[test]
fn break_line() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.break_line(5, 0);
    assert_eq!(buffer.to_string(), "Hello\n world");
}

#[test]
fn join_line() {
    let raw = "Hello\n world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.join_line(0, 0);
    assert_eq!(buffer.to_string(), "Hello world");
}

#[test]
fn insert_5_lines() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.break_line(5, 0);
    buffer.break_line(0, 1);
    buffer.break_line(0, 1);
    buffer.break_line(0, 1);
    buffer.break_line(0, 1);
    assert_eq!(buffer.to_string(), "Hello\n\n\n\n\n world");
}

#[test]
fn join_5_lines() {
    let raw = "Hello\n\n\n\n\n world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.join_line(0, 0);
    buffer.join_line(0, 0);
    buffer.join_line(0, 0);
    buffer.join_line(0, 0);
    buffer.join_line(0, 0);
    assert_eq!(buffer.to_string(), "Hello world");
}

#[test]
fn insert_new_line_at_start() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.break_line(0, 0);
    assert_eq!(buffer.to_string(), "\nHello world");
}

#[test]
fn insert_new_line_at_end() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.break_line(11, 0);
    assert_eq!(buffer.to_string(), "Hello world\n");
}

#[test]
fn insert_anywhere_col() {
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), "");
    buffer.insert_char(5, 0, 'Y');
    assert_eq!(buffer.to_string(), "     Y");
}

#[test]
fn insert_anywhere_line() {
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), "");
    buffer.insert_char(0, 5, 'Y');
    assert_eq!(buffer.to_string(), "\n\n\n\n\nY");
}

#[test]
fn insert_anywhere_cell() {
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), "");
    buffer.insert_char(2, 5, 'Y');
    assert_eq!(buffer.to_string(), "\n\n\n\n\n  Y");
}

#[test]
fn insert_anywhere_cell_10_10() {
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), "");
    buffer.insert_char(10, 10, 'Y');
    assert_eq!(buffer.to_string(), "\n\n\n\n\n\n\n\n\n\n          Y");
}

#[test]
fn breaking_line_anywhere_cell_no_effect_on_nosuchline() {
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), "");
    buffer.break_line(2, 5);
    assert_eq!(buffer.to_string(), "");
}

#[test]
fn lines_2() {
    let raw = "Hello\nworld";
    let buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    assert_eq!(buffer.total_lines(), 2);
    assert_eq!(buffer.line_width(0), Some(5));
    assert_eq!(buffer.line_width(1), Some(5));
    assert_eq!(buffer.line_width(2), None);
}

#[test]
fn cjk() {
    let raw = "Hello 文件系统";
    let buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    assert_eq!(buffer.total_lines(), 1);
    assert_eq!(buffer.line_width(0), Some(14));
}

#[test]
fn insert_end_cjk() {
    let raw = "Hello 文件系统";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_char(13, 0, 'Y');
    assert_eq!(buffer.to_string(), "Hello 文件系统Y");
}

#[test]
fn insert_end_cjk_same_insert_on_13th_or_14th() {
    let raw = "Hello 文件系统";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_char(14, 0, 'Y');
    assert_eq!(buffer.to_string(), "Hello 文件系统Y");
}

#[test]
fn insert_end_cjk_but_not_15th() {
    let raw = "Hello 文件系统";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_char(15, 0, 'Y');
    assert_eq!(buffer.to_string(), "Hello 文件系统 Y");
}

#[test]
fn replace_start() {
    let raw = "Hello";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.replace_char(0, 0, 'Y');
    assert_eq!(buffer.to_string(), "Yello");
}

#[test]
fn replace_middle() {
    let raw = "Hello";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.replace_char(2, 0, 'Y');
    assert_eq!(buffer.to_string(), "HeYlo");
}

#[test]
fn replace_end() {
    let raw = "Hello";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.replace_char(4, 0, 'Y');
    assert_eq!(buffer.to_string(), "HellY");
}

#[test]
fn insert_start() {
    let raw = "Hello";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_char(0, 0, 'Y');
    assert_eq!(buffer.to_string(), "YHello");
}

#[test]
fn insert_middle() {
    let raw = "Hello";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_char(2, 0, 'Y');
    assert_eq!(buffer.to_string(), "HeYllo");
}

#[test]
fn insert_end() {
    let raw = "Hello";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_char(5, 0, 'Y');
    assert_eq!(buffer.to_string(), "HelloY");
}

#[test]
fn test_cut_text() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    let txt = buffer.cut_text(Point2::new(0, 0), Point2::new(4, 0));
    assert_eq!(txt, "Hello");
    assert_eq!(buffer.to_string(), " world");
}

#[test]
fn test_cut_text_multi_line() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    let txt = buffer.cut_text(Point2::new(0, 1), Point2::new(4, 1));
    assert_eq!(txt, "Hello");
    assert_eq!(buffer.to_string(), "before text\n world\nafter text");
}

#[test]
// before text
// Hello world
// after text
//
// before text
// text
fn test_cut_text_2lines_multi_line() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    let txt = buffer.cut_text(Point2::new(0, 1), Point2::new(4, 2));
    assert_eq!(txt, "Hello world\nafter");
    //FIXME: There should only be 1 \n here
    assert_eq!(buffer.to_string(), "before text\n\n text");
}

#[test]
fn test_cut_text_2lines_multi_line_block_mode() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer = TextBuffer::from_str(
        Options {
            use_block_mode: true,
            ..Default::default()
        },
        Context::default(),
        raw,
    );
    let txt = buffer.cut_text(Point2::new(6, 1), Point2::new(10, 2));
    assert_eq!(txt, "world\ntext");
    assert_eq!(buffer.to_string(), "before text\nHello \nafter ");
}

#[test]
fn test_insert_text() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_text(5, 0, "YYYY");
    assert_eq!(buffer.to_string(), "HelloYYYY world");
}

#[test]
fn test_insert_multi_line_text() {
    let raw = "Hello world";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_text(5, 0, "XXXX\nYYYY");
    assert_eq!(buffer.to_string(), "HelloXXXX\nYYYY world");
}

#[test]
fn test_insert_multi_line_text_to_multi_line_text() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer =
        TextBuffer::from_str(Options::default(), Context::default(), raw);
    buffer.insert_text(5, 1, "XXXX\nYYYY");
    assert_eq!(
        buffer.to_string(),
        "before text\nHelloXXXX\nYYYY world\nafter text"
    );
}
