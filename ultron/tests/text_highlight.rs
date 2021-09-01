use ultron::editor::TextHighlight;
#[test]
fn line_length() {
    let raw = "Hello world";
    let canvas = TextHighlight::from_str(raw);
    assert_eq!(canvas.total_lines(), 1);
    assert_eq!(canvas.line_width(0), Some(11));
}

#[test]
fn delete_last_char() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.delete_char(10, 0);
    assert_eq!(canvas.to_string(), "Hello worl");
}

#[test]
fn delete_first_char() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.delete_char(0, 0);
    assert_eq!(canvas.to_string(), "ello world");
}

#[test]
fn delete_5th_char() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.delete_char(5, 0);
    assert_eq!(canvas.to_string(), "Helloworld");
}

#[test]
fn delete_in_2nd_line() {
    let raw = "Hello\nworld\nthere";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.delete_char(0, 2);
    assert_eq!(canvas.to_string(), "Hello\nworld\nhere");
}

#[test]
fn break_line() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.break_line(5, 0);
    assert_eq!(canvas.to_string(), "Hello\n world");
}

#[test]
fn insert_5_lines() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.break_line(5, 0);
    canvas.break_line(0, 1);
    canvas.break_line(0, 1);
    canvas.break_line(0, 1);
    canvas.break_line(0, 1);
    assert_eq!(canvas.to_string(), "Hello\n\n\n\n\n world");
}

#[test]
fn insert_new_line_at_start() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.break_line(0, 0);
    assert_eq!(canvas.to_string(), "\nHello world");
}

#[test]
fn insert_new_line_at_end() {
    let raw = "Hello world";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.break_line(11, 0);
    assert_eq!(canvas.to_string(), "Hello world\n");
}

#[test]
fn insert_anywhere_col() {
    let mut canvas = TextHighlight::from_str("");
    canvas.insert_char(5, 0, 'Y');
    assert_eq!(canvas.to_string(), "     Y");
}

#[test]
fn insert_anywhere_line() {
    let mut canvas = TextHighlight::from_str("");
    canvas.insert_char(0, 5, 'Y');
    assert_eq!(canvas.to_string(), "\n\n\n\n\nY");
}

#[test]
fn lines_2() {
    let raw = "Hello\nworld";
    let canvas = TextHighlight::from_str(raw);
    assert_eq!(canvas.total_lines(), 2);
    assert_eq!(canvas.line_width(0), Some(5));
    assert_eq!(canvas.line_width(1), Some(5));
    assert_eq!(canvas.line_width(2), None);
}

#[test]
fn cjk() {
    let raw = "Hello 文件系统";
    let canvas = TextHighlight::from_str(raw);
    assert_eq!(canvas.total_lines(), 1);
    assert_eq!(canvas.line_width(0), Some(14));
}

#[test]
fn insert_end_cjk() {
    let raw = "Hello 文件系统";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.insert_char(13, 0, 'Y');
    assert_eq!(canvas.to_string(), "Hello 文件系统Y");
}

#[test]
fn insert_end_cjk_same_insert_on_13th_or_14th() {
    let raw = "Hello 文件系统";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.insert_char(14, 0, 'Y');
    assert_eq!(canvas.to_string(), "Hello 文件系统Y");
}

#[test]
fn insert_end_cjk_but_not_15th() {
    let raw = "Hello 文件系统";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.insert_char(15, 0, 'Y');
    assert_eq!(canvas.to_string(), "Hello 文件系统 Y");
}

#[test]
fn replace_start() {
    let raw = "Hello";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.replace_char(0, 0, 'Y');
    assert_eq!(canvas.to_string(), "Yello");
}

#[test]
fn replace_middle() {
    let raw = "Hello";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.replace_char(2, 0, 'Y');
    assert_eq!(canvas.to_string(), "HeYlo");
}

#[test]
fn replace_end() {
    let raw = "Hello";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.replace_char(4, 0, 'Y');
    assert_eq!(canvas.to_string(), "HellY");
}

#[test]
fn insert_start() {
    let raw = "Hello";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.insert_char(0, 0, 'Y');
    assert_eq!(canvas.to_string(), "YHello");
}

#[test]
fn insert_middle() {
    let raw = "Hello";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.insert_char(2, 0, 'Y');
    assert_eq!(canvas.to_string(), "HeYllo");
}

#[test]
fn insert_end() {
    let raw = "Hello";
    let mut canvas = TextHighlight::from_str(raw);
    canvas.insert_char(5, 0, 'Y');
    assert_eq!(canvas.to_string(), "HelloY");
}
