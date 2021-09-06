use ultron::sauron::html::attributes;
use ultron::sauron::html::tags::style;
use ultron::sauron::jss::jss;
use ultron::sauron::prelude::*;
use ultron::Options;
use ultron::TextBuffer;

pub fn render<MSG>(content: &str) -> Node<MSG> {
    let options = Options {
        show_line_numbers: true,
        show_status_line: false,
        show_cursor: false,
        use_spans: true,
        use_for_ssg: true,
    };
    let buffer = TextBuffer::from_str(options, content, "rust");
    page(buffer)
}

pub fn render_to_string(content: &str) -> String {
    render::<()>(content).render_to_string()
}

fn page<MSG>(buffer: TextBuffer) -> Node<MSG> {
    html(
        vec![],
        vec![
            head(
                vec![],
                vec![
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
                ],
            ),
            html::body(vec![], vec![buffer.view()]),
        ],
    )
}
