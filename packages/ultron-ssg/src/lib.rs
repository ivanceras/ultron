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
) -> (Node<MSG>, String) {
    let options = Options {
        show_line_numbers: true,
        show_status_line: false,
        show_cursor: false,
        use_spans: true,
        use_for_ssg: true,
        theme_name: theme_name.map(|s| s.to_string()),
    };
    let buffer = TextBuffer::from_str(options, content, syntax_token);
    let css = buffer.style();
    (page(buffer), css)
}

pub fn render_to_string(
    content: &str,
    syntax_token: &str,
    theme_name: Option<&str>,
) -> String {
    let (node, _css) = render::<()>(content, syntax_token, theme_name);
    node.render_to_string()
}

fn page<MSG>(buffer: TextBuffer) -> Node<MSG> {
    main(
        vec![],
        vec![
            header(
                vec![],
                vec![
                /*
                    meta(
                        vec![
                            content("text/html;charset=utf-8"),
                            attr("http-equiv", "Content-Type"),
                        ],
                        vec![],
                    ),
                    meta(
                        vec![
                            attributes::name("viewport"),
                            content("width=device-width, initial-scale=1"),
                        ],
                        vec![],
                    ),
                    style(
                        vec![r#type("text/css")],
                        vec![text(jss! {
                            "body": {
                                font_family: "monospace",
                                font_size: px(14),
                                cursor: "text",
                                width: percent(100),
                                height: percent(100),
                                margin: 0,
                            }
                        })],
                    ),
                    style(vec![r#type("text/css")], vec![text(buffer.style())]),
                */],
            ),
            article(vec![], vec![buffer.view()]),
        ],
    )
}
