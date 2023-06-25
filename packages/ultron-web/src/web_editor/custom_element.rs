use super::{Msg, WebEditor};
use sauron::{
    dom::{MountAction, MountTarget},
    html::*,
    *,
};
use std::collections::BTreeMap;

impl<XMSG> sauron::CustomElement<Msg> for WebEditor<XMSG>
where
    XMSG: 'static,
{
    fn custom_tag() -> &'static str {
        "ultron-editor"
    }
    fn observed_attributes() -> Vec<&'static str> {
        vec!["content"]
    }

    /// this is called when the attributes in the mount is changed
    fn attribute_changed<DSP>(
        program: &DSP,
        attr_name: &str,
        _old_value: JsValue,
        new_value: JsValue,
    ) where
        DSP: Dispatch<Msg> + Clone + 'static,
    {
        match &*attr_name {
            "content" => {
                if let Some(new_value) = new_value.as_string() {
                    log::info!("value is changed.. {new_value}");
                    program.dispatch(Msg::ValueChanged(new_value));
                }
            }
            _ => (),
        }
    }

    /// This is called when the attributes for the mount is to be set
    /// this is called every after update
    fn attributes_for_mount(&self) -> BTreeMap<String, String> {
        BTreeMap::from_iter([("value".to_string(), self.get_content())])
    }
}

#[wasm_bindgen]
pub struct WebEditorCustomElement {
    program: Program<WebEditor<()>, Msg>,
}

#[wasm_bindgen]
impl WebEditorCustomElement {
    #[wasm_bindgen(constructor)]
    pub fn new(node: JsValue) -> Self {
        use sauron::wasm_bindgen::JsCast;
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

    #[allow(unused)]
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
        WebEditor::<()>::attribute_changed(&self.program, attr_name, old_value, new_value);
    }

    #[wasm_bindgen(method, js_name = connectedCallback)]
    pub fn connected_callback(&mut self) {
        self.program.mount();
        let component_style =
            <WebEditor<()> as Application<Msg>>::style(&self.program.app.borrow());
        self.program.inject_style_to_mount(&component_style);
        self.program.update_dom().expect("must update dom");
    }

    #[wasm_bindgen(method, js_name = disconnectedCallback)]
    pub fn disconnected_callback(&mut self) {}

    #[wasm_bindgen(method, js_name = adoptedCallback)]
    pub fn adopted_callback(&mut self) {}

    fn struct_name() -> &'static str {
        "WebEditorCustomElement"
    }

    pub fn register() {
        sauron::dom::register_custom_element(WebEditor::<()>::custom_tag(), Self::struct_name());
    }
}

pub fn ultron_editor<MSG>(
    attrs: impl IntoIterator<Item = Attribute<MSG>>,
    children: impl IntoIterator<Item = Node<MSG>>,
) -> Node<MSG> {
    WebEditorCustomElement::register();
    html_element(None, WebEditor::<()>::custom_tag(), attrs, children, true)
}