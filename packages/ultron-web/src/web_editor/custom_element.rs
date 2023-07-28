use super::{Msg, WebEditor};
use sauron::{html::*, *};

impl<XMSG> sauron::WebComponent<Msg> for WebEditor<XMSG>
where
    XMSG: 'static,
{
    fn observed_attributes() -> Vec<&'static str> {
        vec!["value", "syntax", "theme"]
    }

    /// this is called when the attributes in the mount is changed
    fn attribute_changed(
        program: &Program<Self, Msg>,
        attr_name: &str,
        _old_value: Option<String>,
        new_value: Option<String>,
    ) where
        Self: Sized + Application<Msg>,
    {
        match &*attr_name {
            "value" => {
                if let Some(new_value) = new_value {
                    log::info!("value is changed.. {new_value}");
                    program.dispatch(Msg::ChangeValue(new_value));
                }
            }
            "syntax" => {
                if let Some(new_value) = new_value {
                    log::info!("syntax token is changed: {new_value}");
                    program.dispatch(Msg::ChangeSyntax(new_value));
                }
            }
            "theme" => {
                if let Some(new_value) = new_value {
                    log::info!("theme is changed: {new_value}");
                    program.dispatch(Msg::ChangeTheme(new_value));
                }
            }
            _ => (),
        }
    }

    fn connected_callback(&mut self) {}
    fn disconnected_callback(&mut self) {}
    fn adopted_callback(&mut self) {}
}

#[wasm_bindgen]
pub struct WebEditorCustomElement {
    program: Program<WebEditor<()>, Msg>,
}

#[wasm_bindgen]
impl WebEditorCustomElement {
    #[wasm_bindgen(constructor)]
    pub fn new(node: JsValue) -> Self {
        let mount_node: &web_sys::Node = node.unchecked_ref();
        Self {
            program: Program::new(
                WebEditor::<()>::default(),
                mount_node,
                MountAction::Append,
                MountTarget::ShadowRoot,
            ),
        }
    }

    #[allow(unused_variables)]
    #[wasm_bindgen(getter, static_method_of = Self, js_name = observedAttributes)]
    pub fn observed_attributes() -> JsValue {
        let attributes = WebEditor::<()>::observed_attributes();
        serde_wasm_bindgen::to_value(&attributes).expect("convert to value")
    }

    #[wasm_bindgen(method, js_name = attributeChangedCallback)]
    pub fn attribute_changed_callback(
        &self,
        attr_name: &str,
        old_value: JsValue,
        new_value: JsValue,
    ) {
        WebEditor::<()>::attribute_changed(
            &self.program,
            attr_name,
            old_value.as_string(),
            new_value.as_string(),
        );
    }

    #[wasm_bindgen(method, js_name = connectedCallback)]
    pub fn connected_callback(&mut self) {
        self.program.mount();

        let static_style = <WebEditor<()> as Application<Msg>>::stylesheet().join("");
        self.program.inject_style_to_mount(&static_style);
        let dynamic_style =
            <WebEditor<()> as Application<Msg>>::style(&self.program.app.borrow()).join("");
        self.program.inject_style_to_mount(&dynamic_style);

        self.program
            .update_dom(&sauron::dom::Modifier::default())
            .expect("must update dom");
    }

    #[wasm_bindgen(method, js_name = disconnectedCallback)]
    pub fn disconnected_callback(&mut self) {}

    #[wasm_bindgen(method, js_name = adoptedCallback)]
    pub fn adopted_callback(&mut self) {}

    fn struct_name() -> &'static str {
        "WebEditorCustomElement"
    }

    pub fn register() {
        sauron::dom::register_web_component("ultron-editor", Self::struct_name());
    }
}
pub fn register() {
    WebEditorCustomElement::register();
}

pub mod attributes {
    use sauron::html::attributes::{attr, Value};
    use sauron::*;

    pub fn syntax<MSG, V: Into<Value>>(value: V) -> Attribute<MSG> {
        attr("syntax", value)
    }

    pub fn theme<MSG, V: Into<Value>>(value: V) -> Attribute<MSG> {
        attr("theme", value)
    }
}

pub fn ultron_editor<MSG>(
    attrs: impl IntoIterator<Item = Attribute<MSG>>,
    children: impl IntoIterator<Item = Node<MSG>>,
) -> Node<MSG> {
    if !children.into_iter().collect::<Vec<_>>().is_empty() {
        log::warn!("ultron editor ignore the passed children nodes");
    }
    html_element(None, "ultron-editor", attrs, [], true)
}
