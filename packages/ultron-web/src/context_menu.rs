use sauron::{html::attributes::*, html::events::*, html::units::*, html::*, *};
use sauron::{Callback, Task};
use ultron_core::nalgebra::Point2;

#[derive(Debug, Clone)]
pub enum Msg {
    SelectAction(MenuAction),
    ShowAt(Point2<i32>),
}

#[derive(Clone)]
pub struct Menu<XMSG> {
    listeners: Vec<Callback<MenuAction, XMSG>>,
    position: Option<Point2<i32>>,
}

impl<XMSG> Default for Menu<XMSG> {
    fn default() -> Self {
        Self {
            listeners: vec![],
            position: None,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum MenuAction {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
}

impl<XMSG> Menu<XMSG> {
    pub fn new() -> Self {
        Self {
            listeners: vec![],
            position: None,
        }
    }
}

impl<XMSG> Menu<XMSG> {
    pub(crate) fn on_activate<F>(mut self, f: F) -> Self
    where
        F: Fn(MenuAction) -> XMSG + 'static,
    {
        self.listeners.push(Callback::from(f));
        self
    }
}

impl<XMSG> Component<Msg, XMSG> for Menu<XMSG> {
    fn init(&mut self) -> Vec<Task<Msg>> {
        vec![]
    }
    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::SelectAction(menu_action) => {
                let xmsgs: Vec<XMSG> = self
                    .listeners
                    .iter()
                    .map(|listener| listener.emit(menu_action))
                    .collect();
                Effects::with_external(xmsgs)
            }
            Msg::ShowAt(point) => {
                self.position = Some(point);
                Effects::none()
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        let pos = self.position.unwrap_or(Point2::new(0, 0));
        div(
            [
                class("context_menu"),
                style! {
                   position: "absolute",
                   top: px(pos.y),
                   left: px(pos.x),
                },
            ],
            [details(
                [open(true)],
                [
                    html::summary([], [text(" ")]),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::Undo))],
                        [text("Undo")],
                    ),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::Redo))],
                        [text("Redo")],
                    ),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::Cut))],
                        [text("Cut")],
                    ),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::Copy))],
                        [text("Copy")],
                    ),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::Paste))],
                        [text("Paste")],
                    ),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::Delete))],
                        [text("Delete")],
                    ),
                    li(
                        [on_click(|_| Msg::SelectAction(MenuAction::SelectAll))],
                        [text("Select All")],
                    ),
                ],
            )],
        )
    }

    fn style(&self) -> Vec<String> {
        vec![jss_pretty! {
            ".context_menu": {
                background_color: "#eee",
                display: "flex",
                flex_direction: "row",
                user_select: "none",
                "-webkit-user-select": "none",
            },

            ".context_menu details": {
                position: "absolute",
                display: "flex",
                width: px(120),
                flex_direction: "column",
                justify_content: "center",
                align_content: "center",
                border: format!("{} solid #ccc",px(1)),
                cursor: "default",
            },

            ".context_menu details[open]": {
                background_color: "#eee",
                border_bottom: 0,
            },

            ".context_menu details summary": {
                list_style: "none",
                outline: "none",
                width: px(120),
                padding: px([5, 5]),
            },

            ".context_menu details summary::-webkit-details-marker": {
                display: "none",
            },

            ".context_menu details[open] summary": {
                border_bottom: format!("{} solid #ccc", px(1)),
            },

            ".context_menu details li": {
                list_style: "none",
                padding: px([5, 5]),
                border_bottom: format!("{} solid #ddd", px(1)),
            },

            ".context_menu details li:hover": {
                background_color: "#ddd",
            },
        }]
    }
}
