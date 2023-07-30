use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sauron::wasm_bindgen::JsCast;
use sauron::wasm_bindgen_futures::JsFuture;
use sauron::{
    events::*,
    html::{attributes::*, units::*, *},
    *,
};
use web_sys::FontFace;

const IOSEVKA_FONT: &[u8] = include_bytes!("../../../fonts/iosevka-fixed-regular.woff2");

#[derive(Clone, Debug)]
pub struct FontSettings {
    pub font_family: String,
    pub font_src: String,
    pub font_size: usize,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            font_family: "Iosevka Fixed".to_string(),
            font_src: format!(
                "url(data:font/woff2;base64,{}) format('woff2')",
                STANDARD.encode(IOSEVKA_FONT)
            ),
            font_size: 14,
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    FontsLoaded,
    FontMeasureMounted(MountEvent),
}
pub struct FontLoader<XMSG> {
    pub settings: FontSettings,
    ready_listener: Vec<Callback<(), XMSG>>,
    mount_element: Option<web_sys::Element>,
    /// if the fonts has been loaded
    is_fonts_loaded: bool,
    /// are the loaded fonts has been measured
    is_fonts_measured: bool,
    pub ch_width: Option<f32>,
    pub ch_height: Option<f32>,
}

impl<XMSG> Default for FontLoader<XMSG> {
    fn default() -> Self {
        Self {
            settings: FontSettings::default(),
            mount_element: None,
            ready_listener: vec![],
            is_fonts_loaded: false,
            is_fonts_measured: false,
            ch_width: None,
            ch_height: None,
        }
    }
}

impl<XMSG> FontLoader<XMSG> {
    pub fn new(settings: &FontSettings) -> Self {
        Self {
            settings: settings.clone(),
            ..Default::default()
        }
    }

    /// add a callback to be called when the fonts has already been loaded and measured
    pub fn on_fonts_ready<F>(&mut self, f: F)
    where
        F: Fn() -> XMSG + 'static,
    {
        self.ready_listener.push(Callback::from(move |_| f()));
    }

    fn measure_font(&self) -> Option<(f32, f32)> {
        self.mount_element.as_ref().map(|elm| {
            let rect = elm.get_bounding_client_rect();
            (rect.width() as f32, rect.height() as f32)
        })
    }

    fn try_measure_font(&mut self) -> Vec<XMSG> {
        if self.is_fonts_loaded {
            if let Some((ch_width, ch_height)) = self.measure_font() {
                self.ch_width = Some(ch_width);
                self.ch_height = Some(ch_height);
                self.is_fonts_measured = true;
                self.ready_listener
                    .iter()
                    .map(|c| {
                        c.emit(())
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_fonts_loaded && self.is_fonts_measured && self.mount_element.is_some()
    }
}

impl<XMSG> Component<Msg, XMSG> for FontLoader<XMSG>
where
    XMSG: 'static,
{
    fn init(&mut self) -> Effects<Msg, XMSG> {
        let font_family = self.settings.font_family.to_owned();
        let font_src = self.settings.font_src.to_owned();
        let font_size = self.settings.font_size;
        Effects::with_local_async([async move {
            let font_set = document().fonts();
            let font_face = FontFace::new_with_str(&font_family, &font_src).expect("font face");
            font_set.add(&font_face).expect("font added");
            // Note: the 14px in-front of the font family is needed for this to work
            // properly
            JsFuture::from(font_set.load(&format!("{} {}", px(font_size), font_family)))
                .await
                .expect("font loaded");
            Msg::FontsLoaded
        }])
    }

    fn update(&mut self, msg: Msg) -> Effects<Msg, XMSG> {
        match msg {
            Msg::FontMeasureMounted(mount_event) => {
                let elm: web_sys::Element = mount_event.target_node.unchecked_into();
                self.mount_element = Some(elm);
                let xmsgs = self.try_measure_font();
                Effects::with_external(xmsgs)
            }
            Msg::FontsLoaded => {
                self.is_fonts_loaded = true;
                let xmsgs = self.try_measure_font();
                Effects::with_external(xmsgs)
            }
        }
    }

    fn view(&self) -> Node<Msg> {
        let font_size = self.settings.font_size;
        let font_family = self.settings.font_family.to_owned();
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
                                font_family: font_family,
                            },
                            on_mount(Msg::FontMeasureMounted),
                        ],
                        [text("0")],
                    )],
                ),
            ],
        )
    }

    fn stylesheet() -> Vec<String> {
        vec![]
    }

    fn style(&self) -> Vec<String> {
        vec![]
    }
}
