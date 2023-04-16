use nalgebra::Point2;
use ultron_core::TextBuffer;

#[test]
fn line_length() {
    let raw = "Hello world";
    let buffer = TextBuffer::from_str(raw);
    assert_eq!(buffer.total_lines(), 1);
    assert_eq!(buffer.line_width(0), 11);
}

#[test]
fn cjk_length() {
    let raw = "CJK文件";
    let buffer = TextBuffer::from_str(raw);
    assert_eq!(buffer.total_chars(), 5);
    assert_eq!(buffer.total_lines(), 1);
    assert_eq!(buffer.line_width(0), 7);
}

#[test]
fn test_get_char() {
    let raw = "Hello world";
    let buffer = TextBuffer::from_str(raw);
    assert_eq!(buffer.get_char(6, 0), Some('w'));
}

#[test]
fn test_get_text() {
    let raw = "Hello world";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(0, 0), Point2::new(4, 0));
    assert_eq!(txt, "Hello");
}

#[test]
fn test_get_text_world() {
    let raw = "Hello world";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(6, 0), Point2::new(10, 0));
    assert_eq!(txt, "world");
}

#[test]
fn test_get_text_with_cjk() {
    let raw = "Hello 文件系统 world";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(8, 0), Point2::new(10, 0));
    assert_eq!(txt, "件系");
}

#[test]
fn test_cut_text_with_cjk() {
    let raw = "Hello 文件系统 world";
    let mut buffer = TextBuffer::from_str(raw);
    let txt = buffer.cut_text(Point2::new(8, 0), Point2::new(10, 0));
    assert_eq!(txt, "件系");
    assert_eq!(buffer.to_string(), "Hello 文统 world");
}

#[test]
fn test_get_text_with_cjk_world() {
    let raw = "Hello 文件系统 world";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(15, 0), Point2::new(19, 0));
    assert_eq!(txt, "world");
}

#[test]
fn test_cut_text_with_cjk_world() {
    let raw = "Hello 文件系统 world";
    let mut buffer = TextBuffer::from_str(raw);
    let txt = buffer.cut_text(Point2::new(15, 0), Point2::new(19, 0));
    assert_eq!(txt, "world");
    assert_eq!(buffer.to_string(), "Hello 文件系统 ");
}

#[test]
fn test_get_text_multi_line_world() {
    let raw = "Hello\nworld and\neverywhere";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(0, 1), Point2::new(4, 1));
    assert_eq!(txt, "world");
}

#[test]
fn test_get_text_multi_line_and() {
    let raw = "Hello\nworld and\neverywhere";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(6, 1), Point2::new(8, 1));
    assert_eq!(txt, "and");
}

#[test]
fn test_get_text_multi_line_and_every() {
    let raw = "Hello\nworld and\neverywhere";
    let buffer = TextBuffer::from_str(raw);
    let txt = buffer.get_text(Point2::new(6, 1), Point2::new(4, 2));
    assert_eq!(txt, "and\nevery");
}

#[test]
fn delete_last_char() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.delete_char(10, 0);
    assert_eq!(buffer.to_string(), "Hello worl");
}

#[test]
fn delete_first_char() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.delete_char(0, 0);
    assert_eq!(buffer.to_string(), "ello world");
}

#[test]
fn delete_5th_char() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.delete_char(5, 0);
    assert_eq!(buffer.to_string(), "Helloworld");
}

#[test]
fn delete_in_2nd_line() {
    let raw = "Hello\nworld\nthere";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.delete_char(0, 2);
    assert_eq!(buffer.to_string(), "Hello\nworld\nhere");
}

#[test]
fn break_line() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(5, 0);
    assert_eq!(buffer.to_string(), "Hello\n world");
}

#[test]
fn break_line_then_insert_1() {
    let raw = "Hello world\n\nHowdy";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(0, 1);
    assert_eq!(buffer.to_string(), "Hello world\n\n\nHowdy");
    buffer.insert_char(0, 1, '1');
    assert_eq!(buffer.to_string(), "Hello world\n1\n\nHowdy");
}

#[test]
fn break_line_in_non_existing_cell() {
    let raw = "Hello world\n\nHowdy";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(2, 1);
    assert_eq!(buffer.to_string(), "Hello world\n  \n\nHowdy");
    buffer.insert_char(0, 2, '1');
    assert_eq!(buffer.to_string(), "Hello world\n  \n1\nHowdy");
}

#[test]
fn break_line_then_insert_1_below() {
    let raw = "Hello world\n\nHowdy";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(0, 1);
    assert_eq!(buffer.to_string(), "Hello world\n\n\nHowdy");
    buffer.insert_char(0, 2, '1');
    assert_eq!(buffer.to_string(), "Hello world\n\n1\nHowdy");
}

#[test]
fn break_line_then_insert_2_below() {
    let raw = "Hello world\n\nHowdy";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(0, 1);
    buffer.break_line(0, 1);
    assert_eq!(buffer.to_string(), "Hello world\n\n\n\nHowdy");
    buffer.insert_char(0, 3, '2');
    assert_eq!(buffer.to_string(), "Hello world\n\n\n2\nHowdy");
}

#[test]
fn join_line() {
    let raw = "Hello\n world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.join_line(0, 0);
    assert_eq!(buffer.to_string(), "Hello world");
}

#[test]
fn ensure_line_exist5() {
    let mut buffer = TextBuffer::from_str("");
    buffer.ensure_line_exist(5);
    assert_eq!(buffer.total_lines(), 6);
}

#[test]
fn ensure_cell_exist5_2() {
    let mut buffer = TextBuffer::from_str("");
    buffer.ensure_cell_exist(5, 2);
    assert_eq!(buffer.to_string(), "\n\n      ");
}

#[test]
fn ensure_cell_exist0_0() {
    let mut buffer = TextBuffer::from_str("");
    buffer.ensure_cell_exist(0, 0);
    assert_eq!(buffer.to_string(), " ");
}

#[test]
fn insert_5_lines() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
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
    let mut buffer = TextBuffer::from_str(raw);
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
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(0, 0);
    assert_eq!(buffer.to_string(), "\nHello world");
}

#[test]
fn insert_new_line_at_end() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.break_line(11, 0);
    assert_eq!(buffer.to_string(), "Hello world\n");
}

#[test]
fn insert_anywhere_col() {
    let mut buffer = TextBuffer::from_str("");
    buffer.insert_char(5, 0, 'Y');
    assert_eq!(buffer.to_string(), "     Y");
}

#[test]
fn insert_anywhere_line() {
    let mut buffer = TextBuffer::from_str("");
    buffer.insert_char(0, 5, 'Y');
    assert_eq!(buffer.to_string(), "\n\n\n\n\nY");
}

#[test]
fn insert_anywhere_cell() {
    let mut buffer = TextBuffer::from_str("");
    buffer.insert_char(2, 5, 'Y');
    assert_eq!(buffer.to_string(), "\n\n\n\n\n  Y");
}

#[test]
fn insert_anywhere_cell_10_10() {
    let mut buffer = TextBuffer::from_str("");
    buffer.insert_char(10, 10, 'Y');
    assert_eq!(buffer.to_string(), "\n\n\n\n\n\n\n\n\n\n          Y");
}

#[test]
fn breaking_line_anywhere_will_make_lines_on_it() {
    let mut buffer = TextBuffer::from_str("");
    buffer.break_line(2, 5);
    assert_eq!(buffer.to_string(), "\n\n\n\n\n  \n");
}

#[test]
fn lines_2() {
    let raw = "Hello\nworld";
    let buffer = TextBuffer::from_str(raw);
    assert_eq!(buffer.total_lines(), 2);
    assert_eq!(buffer.line_width(0), 5);
    assert_eq!(buffer.line_width(1), 5);
    assert_eq!(buffer.line_width(2), 0);
}

#[test]
fn cjk() {
    let raw = "Hello 文件系统";
    let buffer = TextBuffer::from_str(raw);
    assert_eq!(buffer.total_lines(), 1);
    assert_eq!(buffer.line_width(0), 14);
}

#[test]
fn insert_end_cjk() {
    let raw = "Hello 文件系统";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_char(14, 0, 'Y');
    assert_eq!(buffer.to_string(), "Hello 文件系统Y");
}

#[test]
fn insert_end_cjk_same_insert_on_13th_or_14th() {
    let raw = "Hello 文件系统";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_char(14, 0, 'Y');
    assert_eq!(buffer.to_string(), "Hello 文件系统Y");
}

#[test]
fn insert_end_cjk_but_not_15th() {
    let raw = "Hello 文件系统";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_char(15, 0, 'Y');
    assert_eq!(buffer.to_string(), "Hello 文件系统 Y");
}

#[test]
fn replace_start() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.replace_char(0, 0, 'Y');
    assert_eq!(buffer.to_string(), "Yello");
}

#[test]
fn replace_middle() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.replace_char(2, 0, 'Y');
    assert_eq!(buffer.to_string(), "HeYlo");
}

#[test]
fn replace_end() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.replace_char(4, 0, 'Y');
    assert_eq!(buffer.to_string(), "HellY");
}

#[test]
fn replace_char_on_next_page() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.replace_char(4, 30, 'Y');
    assert_eq!(
        buffer.to_string(),
        "Hello\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n    Y"
    );
}

#[test]
fn insert_start() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_char(0, 0, 'Y');
    assert_eq!(buffer.to_string(), "YHello");
}

#[test]
fn insert_middle() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_char(2, 0, 'Y');
    assert_eq!(buffer.to_string(), "HeYllo");
}

#[test]
fn insert_end() {
    let raw = "Hello";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_char(5, 0, 'Y');
    assert_eq!(buffer.to_string(), "HelloY");
}

#[test]
fn test_cut_text() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    let txt = buffer.cut_text(Point2::new(0, 0), Point2::new(4, 0));
    assert_eq!(txt, "Hello");
    assert_eq!(buffer.to_string(), " world");
}

#[test]
fn test_cut_text_multi_line() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer = TextBuffer::from_str(raw);
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
    let mut buffer = TextBuffer::from_str(raw);
    let txt = buffer.cut_text(Point2::new(0, 1), Point2::new(4, 2));
    assert_eq!(txt, "Hello world\nafter");
    //FIXME: There should only be 1 \n here
    assert_eq!(buffer.to_string(), "before text\n\n text");
}

// before text
// Hello world
// after text
//
// world
// text
//
// before text
// Hello
// after
//
#[test]
fn test_cut_text_2lines_multi_line_block_mode() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer = TextBuffer::from_str(raw);
    let txt = buffer.cut_text(Point2::new(6, 1), Point2::new(10, 2));
    assert_eq!(txt, "world\ntext");
    assert_eq!(buffer.to_string(), "before text\nHello \nafter ");
}

#[test]
fn test_insert_text() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_text(5, 0, "YYYY");
    assert_eq!(buffer.to_string(), "HelloYYYY world");
}

#[test]
fn test_insert_multi_line_text() {
    let raw = "Hello world";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_text(5, 0, "XXXX\nYYYY");
    assert_eq!(buffer.to_string(), "HelloXXXX\nYYYY world");
}

#[test]
fn test_insert_multi_line_text_to_multi_line_text() {
    let raw = "before text\nHello world\nafter text";
    let mut buffer = TextBuffer::from_str(raw);
    buffer.insert_text(5, 1, "XXXX\nYYYY");
    assert_eq!(
        buffer.to_string(),
        "before text\nHelloXXXX\nYYYY world\nafter text"
    );
}

#[test]
fn test_get_text_block_mode() {
    let raw = "0000\n01234 Hello 5678\n01234 world 5678\n01234 wazup 5678\n0000";
    let buffer = TextBuffer::from_str(raw);
    let selection = buffer.get_text_block_mode(Point2::new(6, 1), Point2::new(10, 3));
    assert_eq!(selection, "Hello\nworld\nwazup");
    assert_eq!(raw, buffer.to_string());
}

#[test]
fn test_cut_text_block_mode() {
    let raw = "0000\n01234 Hello 5678\n01234 world 5678\n01234 wazup 5678\n0000";
    let mut buffer = TextBuffer::from_str(raw);
    let selection = buffer.cut_text_block_mode(Point2::new(6, 1), Point2::new(10, 3));
    assert_eq!(selection, "Hello\nworld\nwazup");
    assert_eq!(
        buffer.to_string(),
        "0000\n01234  5678\n01234  5678\n01234  5678\n0000"
    );
}
