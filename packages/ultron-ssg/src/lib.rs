#![deny(warnings)]
use css_colors::rgba;
use css_colors::Color as ColorTrait;
use css_colors::RGBA;
use sauron::*;
use ultron_syntaxes_themes::Color;
use ultron_syntaxes_themes::Style;
use ultron_syntaxes_themes::TextHighlighter;
use ultron_syntaxes_themes::Theme;

const COMPONENT_NAME: &str = "ssg";

#[derive(Debug, Default)]
struct Options {
    show_line_numbers: bool,
}

struct CodeViewer {
    options: Options,
    lines: Vec<String>,
    active_theme: &'static Theme,
}

impl CodeViewer {
    fn view<MSG>(&self, text_highlighter: &mut TextHighlighter) -> Node<MSG> {
        let class_ns = |class_names| class_namespaced(COMPONENT_NAME, class_names);
        let code_attributes = [
            class_ns("code"),
            style! {
                background: self.theme_background().to_css(),
                font_family: "monospace",
            },
        ];

        let rendered_lines = self.lines.iter().enumerate().map(|(line_index, line)| {
            let hl_line = text_highlighter
                .highlight_line(line)
                .expect("must highlight");
            div([class_ns("line"), style! {line_height: 1}], {
                [self.view_line_number(line_index + 1)]
                    .into_iter()
                    .chain(self.view_highlighted_line(&hl_line))
                    .collect::<Vec<_>>()
            })
        });

        div(code_attributes, rendered_lines)
    }

    fn view_highlighted_line<MSG>(&self, line: &[(Style, &str)]) -> Vec<Node<MSG>> {
        line.iter()
            .map(|(style, range)| {
                let foreground = to_rgba(style.foreground).to_css();
                span([style! { color: foreground }], [text(range)])
            })
            .collect()
    }

    fn view_line_number<MSG>(&self, line_number: usize) -> Node<MSG> {
        let class_ns = |class_names| class_namespaced(COMPONENT_NAME, class_names);
        view_if(
            self.options.show_line_numbers,
            span(
                [
                    class_ns("number"),
                    style! {
                        user_select: "none",
                        "-webkit-user-select": "none",
                        display: "inline-block",
                        background_color: self.gutter_background().to_css(),
                        color: self.gutter_foreground().to_css(),
                        width: ch(self.numberline_wide()),
                        padding_right: ch(self.numberline_padding_wide()),
                    },
                ],
                [text(line_number)],
            ),
        )
    }

    fn numberline_wide(&self) -> usize {
        self.lines.len().to_string().len()
    }

    fn theme_background(&self) -> RGBA {
        let default = rgba(255, 255, 255, 1.0);
        self.active_theme
            .settings
            .background
            .map(to_rgba)
            .unwrap_or(default)
    }

    fn gutter_background(&self) -> RGBA {
        let default = rgba(0, 0, 0, 1.0);
        self.active_theme
            .settings
            .gutter
            .map(to_rgba)
            .unwrap_or(default)
    }

    fn gutter_foreground(&self) -> RGBA {
        let default = rgba(0, 0, 0, 1.0);
        self.active_theme
            .settings
            .gutter_foreground
            .map(to_rgba)
            .unwrap_or(default)
    }

    pub(crate) fn numberline_padding_wide(&self) -> usize {
        1
    }
}

pub fn to_rgba(color: Color) -> RGBA {
    let Color { r, g, b, a } = color;
    css_colors::rgba(r, g, b, a as f32 / 255.0)
}

pub fn render<MSG>(content: &str, syntax_token: &str, theme_name: Option<&str>) -> Node<MSG>
where
    MSG: 'static,
{
    let options = Options {
        show_line_numbers: true,
    };

    let mut text_highlighter = TextHighlighter::default();
    if let Some(theme_name) = theme_name {
        text_highlighter.select_theme(theme_name);
    }
    text_highlighter.set_syntax_token(syntax_token);

    let viewer = CodeViewer {
        options,
        lines: content.lines().map(|s| s.to_string()).collect(),
        active_theme: text_highlighter.active_theme(),
    };
    viewer.view(&mut text_highlighter)
}

pub fn render_to_string(content: &str, syntax_token: &str, theme_name: Option<&str>) -> String {
    let node = render::<()>(content, syntax_token, theme_name);
    node.render_to_string()
}
