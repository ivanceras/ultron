#![deny(warnings)]

use ultron::{
    sauron::{html::tags::style, prelude::*},
    Editor, Options,
};

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
    let editor = Editor::from_str(options, content);
    page(editor)
}

pub fn render_to_string(
    content: &str,
    syntax_token: &str,
    theme_name: Option<&str>,
) -> String {
    let node = render::<()>(content, syntax_token, theme_name);
    node.render_to_string()
}

fn page<MSG>(editor: Editor<MSG>) -> Node<MSG> {
    main(
        [],
        [
            header([], [style([r#type("text/css")], [text(editor.style())])]),
            article([], [editor.plain_view()]),
        ],
    )
}
