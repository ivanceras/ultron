#![deny(warnings)]

use ultron::sauron::html::tags::style;
use ultron::sauron::prelude::*;
use ultron::Options;
use ultron::TextBuffer;

pub fn render<MSG>(
    content: &str,
    syntax_token: &str,
    theme_name: Option<&str>,
) -> Node<MSG> {
    let options = Options {
        show_line_numbers: true,
        show_status_line: false,
        show_cursor: false,
        use_spans: true,
        use_for_ssg: true,
        theme_name: theme_name.map(|s| s.to_string()),
        syntax_token: syntax_token.to_string(),
        ..Default::default()
    };
    let buffer = TextBuffer::from_str(options, content);
    page(buffer)
}

pub fn render_to_string(
    content: &str,
    syntax_token: &str,
    theme_name: Option<&str>,
) -> String {
    let node = render::<()>(content, syntax_token, theme_name);
    node.render_to_string()
}

fn page<MSG>(buffer: TextBuffer) -> Node<MSG> {
    main(
        [],
        [
            header([], [style([r#type("text/css")], [text(buffer.style())])]),
            article([], [buffer.view()]),
        ],
    )
}
