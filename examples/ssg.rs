use sauron::html::tags::style;
use sauron::jss::jss;
use sauron::prelude::*;
use ultron::Editor;
use ultron::Options;

fn main() {
    let content = include_str!("../test_data/hello.rs");
    let editor =
        Editor::<()>::from_str(content, "rust").with_options(Options {
            show_line_numbers: true,
            show_status_line: false,
            show_cursor: false,
            use_spans: true,
        });
    let html = page(editor).render_to_string();
    std::fs::create_dir_all("out").expect("must create dir");
    std::fs::write("out/hello.html", html);
}

fn page(editor: Editor<()>) -> Node<ultron::editor::Msg> {
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
                    style(
                        vec![r#type("text/css")],
                        vec![text(jss! {
                            "body": {
                                font_family: "monospace"
                            }
                        })],
                    ),
                    style(vec![r#type("text/css")], vec![text(editor.style())]),
                ],
            ),
            html::body(vec![], vec![editor.view()]),
        ],
    )
}
