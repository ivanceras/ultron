#![deny(warnings)]

use ultron_web::{
    sauron::{html::attributes::r#type, html::tags::style, html::text, *},
    Options, WebEditor,
};

pub fn render<MSG>(content: &str, syntax_token: &str, theme_name: Option<&str>) -> Node<MSG> {
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
    let web_editor = WebEditor::from_str(options, content);
    page(web_editor)
}

pub fn render_to_string(content: &str, syntax_token: &str, theme_name: Option<&str>) -> String {
    let node = render::<()>(content, syntax_token, theme_name);
    node.render_to_string()
}

fn page<MSG>(web_editor: WebEditor<MSG>) -> Node<MSG> {
    main(
        [],
        [
            header(
                [],
                [style([r#type("text/css")], [text(web_editor.style())])],
            ),
            article([], [web_editor.plain_view()]),
        ],
    )
}
