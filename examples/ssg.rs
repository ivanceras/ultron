use sauron::html::attributes;
use sauron::html::tags::style;
use sauron::jss::jss;
use sauron::prelude::*;
use ultron::Options;
use ultron::TextBuffer;

fn main() {
    let content = include_str!("../test_data/hello.rs");
    let options = Options {
        show_line_numbers: true,
        show_status_line: false,
        show_cursor: false,
        use_spans: true,
    };
    let buffer = TextBuffer::from_str(options, content, "rust");
    let html = page(buffer).render_to_string();
    std::fs::create_dir_all("out").expect("must create dir");
    std::fs::write("out/hello.html", html);
}

fn page(buffer: TextBuffer) -> Node<()> {
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
