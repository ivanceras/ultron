use super::*;
use syntect::highlighting::Color;

impl Editor {
    pub fn generate_style(&self) -> String {
        let selection_bg = if let Some(selection_bg) = self.selection_background() {
            selection_bg
        } else {
            Color {
                r: 100,
                g: 100,
                b: 100,
                a: 100,
            }
        };

        let cursor_color = if let Some(cursor_color) = self.cursor_color() {
            cursor_color
        } else {
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }
        };

        let css = jss_ns!(COMPONENT_NAME,{
            ".": {
                "user-select": "none",
                "-webkit-user-select": "none",
                "position": "relative",
                "font-size": px(14),
                "cursor": "text",
            },

            // paste area hack, we don't want to use
            // the clipboard read api, since it needs permission from the user
            // create a textarea instead, where it is focused all the time
            // so, pasting will be intercepted from this textarea
            ".paste_area": {
                "resize": "none",
                //"width": 0, //if width is 0, it will not work in chrome
                "height": 0,
                "position": "sticky",
                "top": 0,
                "left": 0,
                "padding": 0,
                "border":0,
            },

            ".code": {
                "position": "relative",
            },

            ".line_block": {
                "display": "block",
            },

            // number and line
            ".number__line": {
                "display": "flex",
            },

            // numbers
            ".number": {
                "flex": "none", // dont compress the numbers
                "text-align": "right",
                "background-color": "cyan",
                "padding-right": ex(1),
            },
            ".number_wide1 .number": {
                "width": px(1 * CH_WIDTH),
            },
            // when line number is in between: 10 - 99
            ".number_wide2 .number": {
                "width": px(2 * CH_WIDTH),
            },
            // when total lines is in between 100 - 999
            ".number_wide3 .number": {
                "width": px(3 * CH_WIDTH),
            },
            // when total lines is in between 1000 - 9000
            // we don't support beyond this
            ".number_wide4 .number": {
                "width": px(4 * CH_WIDTH),
            },

            // line content
            ".line": {
                "display": "flex",
                "flex": "none", // dont compress lines
            },

            ".filler": {
                //"background-color": "#eee",
                "width": percent(100),
            },

            ".line_focused": {
                "background-color": "pink",
            },

            ".range": {
                "display": "flex",
                "flex": "none",
            },

            ".line .ch": {
                "width": px(CH_WIDTH),
                "height": px(CH_HEIGHT),
                "font-family": "monospace",
                "font-stretch": "ultra-condensed",
                "font-variant-numeric": "slashed-zero",
                "font-kerning": "none",
                "font-size-adjust": "none",
                "font-optical-sizing": "none",
                "position": "relative",
                "overflow": "hidden",
                "align-items": "center",
            },

            ".ch.selected": {
                "background-color": Self::convert_rgba(selection_bg),
            },

            ".ch .cursor": {
               "position": "absolute",
               "left": 0,
               "width" : px(CH_WIDTH),
               "height": px(CH_HEIGHT),
               "background-color": Self::convert_rgba(cursor_color),
               //"border": "1px solid red",
               //"margin": "-1px",
               "display": "inline",
               "animation": "cursor_blink-anim 1000ms step-end infinite",
               //"z-index": 1,
            },

            ".ch.wide2 .cursor": {
                "width": px(2 * CH_WIDTH),
            },

            // i-beam cursor
            ".thin_cursor .cursor": {
                "width": px(2),
            },

            ".thin_cursor .wide2 .cursor": {
                "width": px(2 * CH_WIDTH),
            },

            ".block_cursor .cursor": {
                "width": px(CH_WIDTH),
            },

            ".line .ch.wide2": {
                "width": px(2 * CH_WIDTH),
                "font-size": px(12),
            },


            ".status": {
                "background-color": "blue",
                "position": "sticky",
                "bottom": 0,
                "display": "flex",
                "flex-direction": "flex-end",
            },

            "@keyframes cursor_blink-anim": {
              "50%": {
                "background-color": "transparent",
                "border-color": "transparent",
              }
            },

        });
        css
    }
}
