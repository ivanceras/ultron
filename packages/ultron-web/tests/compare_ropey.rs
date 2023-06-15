use ropey::Rope;
use std::time::Instant;
use ultron_core::{Ch, Style, TextEdit, TextHighlighter};

static CODE: &str = include_str!("../src/web_editor.rs");

pub fn text_edit_highlighting() {
    let mut text_highlighter = TextHighlighter::default();
    text_highlighter.set_syntax_token("rust");
    let text_edit = TextEdit::from_str(CODE);
    let _result: Vec<Vec<(Style, Vec<Ch>)>> = text_edit
        .lines()
        .iter()
        .map(|line| {
            text_highlighter
                .highlight_line(line)
                .expect("must highlight")
                .into_iter()
                .map(|(style, line)| (style, line.chars().map(Ch::new).collect()))
                .collect()
        })
        .collect();
}

pub fn ropey_highlighting() {
    let mut text_highlighter = TextHighlighter::default();
    text_highlighter.set_syntax_token("rust");

    let rope = Rope::from_str(CODE);

    let _result: Vec<Vec<(Style, Vec<Ch>)>> = rope
        .lines()
        .map(|line| {
            let line_str = String::from_iter(line.chars());
            text_highlighter
                .highlight_line(&line_str)
                .expect("must highlight")
                .into_iter()
                .map(|(style, line)| (style, line.chars().map(Ch::new).collect()))
                .collect()
        })
        .collect();
}

// This is a simple test to determine whether ropey or a simple Vec<Vec<Char>> is much faster to
// Tested on my machine: Intel(R) Core(TM) i7-3770K CPU @ 3.50GHz
// ropey took 2421ms
// text edit took: 1506ms
#[test]
fn text_edit_is_faster_than_ropey() {
    let t1 = Instant::now();
    ropey_highlighting();
    let ropey_took = t1.elapsed().as_millis();
    println!("ropey took {}ms", ropey_took);
    let t2 = Instant::now();
    text_edit_highlighting();
    let textedit_took = t2.elapsed().as_millis();
    println!("text edit took: {}ms", textedit_took);

    assert!(textedit_took < ropey_took);
}
