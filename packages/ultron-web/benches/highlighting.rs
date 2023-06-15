#![feature(test)]

extern crate test;
use ropey::Rope;
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

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn benchmark_textedit(b: &mut Bencher) {
        b.iter(|| text_edit_highlighting());
    }

    #[bench]
    fn benchmark_ropey(b: &mut Bencher) {
        b.iter(|| ropey_highlighting());
    }
}
