#![allow(unused)]
use crate::wasm_bindgen_futures::spawn_local;
use crate::wasm_bindgen_futures::JsFuture;
use sauron::{
    dom::{Callback, Task},
    html::attributes::*,
    html::events::*,
    html::*,
    *,
};
use wasm_bindgen::JsCast;
use web_sys::FontFace;

pub enum Msg {
    FontsLoaded,
    FontMeasureMounted(MountEvent),
}
#[derive(Default)]
pub struct FontLoader<XMSG> {
    // font size in px
    font_size: f32,
    // name of the font to be used in reference in the document
    font_name: String,
    // the url of the font
    font_url: String,
    fonts_ready_listener: Vec<Callback<(), XMSG>>,
    font_measure_element: Option<web_sys::Element>,
    /// if the fonts has been loaded
    is_fonts_loaded: bool,
    /// are the loaded fonts has been measured
    is_fonts_measured: bool,
    pub ch_width: Option<f32>,
    pub ch_height: Option<f32>,
}

impl<XMSG> FontLoader<XMSG> {
    pub fn new(font_size: f32, font_name: &str, font_url: &str) -> Self {
        Self {
            font_size,
            font_name: font_name.to_string(),
            font_url: font_url.to_string(),
            font_measure_element: None,
            fonts_ready_listener: vec![],
            is_fonts_loaded: false,
            is_fonts_measured: false,
            ch_width: None,
            ch_height: None,
        }
    }

    /// add a callback to be called when the fonts has already been loaded and measured
    pub fn on_fonts_ready<F>(&mut self, f: F)
    where
        F: Fn(()) -> XMSG + 'static,
    {
        self.fonts_ready_listener.push(Callback::from(f));
    }

    fn measure_font(&self) -> Option<(f32, f32)> {
        self.font_measure_element.as_ref().map(|font_elm| {
            let rect = font_elm.get_bounding_client_rect();
            (rect.width() as f32, rect.height() as f32)
        })
    }

    fn try_measure_font(&mut self) -> Vec<XMSG> {
        if self.is_fonts_loaded {
            if let Some((ch_width, ch_height)) = self.measure_font() {
                self.ch_width = Some(ch_width);
                self.ch_height = Some(ch_height);
                log::info!(
                    "font width: {:?}, height: {:?}",
                    self.ch_width,
                    self.ch_height
                );
                self.is_fonts_measured = true;
                self.fonts_ready_listener
                    .iter()
                    .map(|c| c.emit(()))
                    .collect()
            } else {
                log::warn!("font measure element hasn't been mounted yet");
                vec![]
            }
        } else {
            log::warn!("fonts hasn't been loaded yet");
            vec![]
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_fonts_loaded && self.is_fonts_measured
    }
}

impl<XMSG> Component<Msg, XMSG> for FontLoader<XMSG>
where
    XMSG: 'static,
{
    fn init(&mut self) -> Vec<Task<Msg>> {
        log::info!("initializing font loader");
        let font_name = self.font_name.clone();
        let font_url = self.font_url.clone();
        let font_size = self.font_size;
        vec![Task::new(async move{
            let font_set = document().fonts();
            let font_face = FontFace::new_with_str(&font_name, &font_url)
                .expect("font face");
            font_set.add(&font_face);
            // Note: the 14px in-front of the font family is needed for this to work
            // properly
            JsFuture::from(font_set.load(&format!("{} {}", px(font_size),font_name))).await;
            log::info!("awaited the fonts loading...");
            Msg::FontsLoaded
        })
        ]
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::FontMeasureMounted(mount_event) => {
                let font_status = document().fonts().status();
                log::info!("font status: {font_status:?}");
                log::info!("font measure is mounted");
                let font_elm: web_sys::Element = mount_event.target_node.unchecked_into();
                self.font_measure_element = Some(font_elm);
                log::info!("measure font: {:?}", self.measure_font());
                let xmsgs = self.try_measure_font();
                Effects::with_external(xmsgs)
            }
            Msg::FontsLoaded => {
                self.is_fonts_loaded = true;
                log::info!("Fonts are now loaded...");
                let xmsgs = self.try_measure_font();
                Effects::with_external(xmsgs)
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        let font_size = self.font_size;
        let font_name = self.font_name.clone();
        pre(
            [],
            [
                text("font loader is loading"),
                code(
                    [],
                    [span(
                        [
                            class("font_measure"),
                            style! {
                                font_size: px(font_size),
                                font_family: font_name,
                            },
                            on_mount(Msg::FontMeasureMounted),
                        ],
                        [text("0")],
                    )],
                ),
            ],
        )
    }

    fn style(&self) -> Vec<String> {
        vec![]
    }
}
