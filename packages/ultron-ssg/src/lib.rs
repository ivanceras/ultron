//#![deny(warnings)]

use ultron::sauron::html::attributes;
use ultron::sauron::html::tags::style;
use ultron::sauron::jss::jss;
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
    };
    let buffer = TextBuffer::from_str(options, content, syntax_token);
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
        vec![],
        vec![
            header(
                vec![],
                vec![style(
                    vec![r#type("text/css")],
                    vec![text(buffer.style())],
                )],
            ),
            article(vec![], vec![buffer.view()]),
        ],
    )
}
