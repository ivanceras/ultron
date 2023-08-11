use ropey::Rope;
use std::time::Instant;
use ultron_core::{Ch, Style, TextEdit, TextHighlighter};

static CODE: &str = include_str!("../src/web_editor.rs");

pub fn text_edit_highlighting() -> usize {
    let mut text_highlighter = TextHighlighter::default();
    text_highlighter.set_syntax_token("rust");
    let text_edit = TextEdit::new_from_str(CODE);
    let result: Vec<Vec<(Style, Vec<Ch>)>> = text_edit
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
    result.len()
}

pub fn ropey_highlighting() -> usize {
    let mut text_highlighter = TextHighlighter::default();
    text_highlighter.set_syntax_token("rust");

    let rope = Rope::from_str(CODE);

    let result: Vec<Vec<(Style, Vec<Ch>)>> = rope
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
    result.len()
}

// This is a simple test to determine whether ropey or a simple Vec<Vec<Char>> is much faster to
// Tested on: Intel(R) Core(TM) i7-3770K CPU @ 3.50GHz
// - ropey : 2421ms
// - TextEdit: 1506ms
//
// on AMD® Ryzen 5 5600h
//
// `cargo test`:
// took: 1814ms highlighting 1632 lines iterated from ropey
// took: 1077ms highlighting 1632 lines iterated from TextEdit
//
// `cargo test --release`
// took: 140ms highlighting 1632 lines iterated from ropey
// took: 79ms highlighting 1632 lines iterated from TextEdit
//
//
#[test]
fn text_edit_is_faster_than_ropey() {
    let t1 = Instant::now();
    let len = ropey_highlighting();
    let ropey_took = t1.elapsed().as_millis();
    println!("took: {ropey_took}ms highlighting {len} lines iterated from ropey");
    let t2 = Instant::now();
    text_edit_highlighting();
    let textedit_took = t2.elapsed().as_millis();
    println!("took: {textedit_took}ms highlighting {len} lines iterated from TextEdit");

    assert!(textedit_took < ropey_took);
}
