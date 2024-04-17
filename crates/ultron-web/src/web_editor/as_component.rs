use super::{Msg, WebEditor};
use sauron::dom::DomNode;
use sauron::*;

impl StatefulComponent for WebEditor<()> {
    /// this is called when the attributes in the mount is changed
    fn attribute_changed(
        &mut self,
        attr_name: &str,
        new_value: Vec<DomAttrValue>,
    ) 
    {
        match attr_name {
            "value" => {
                if let Some(new_value) = new_value[0].as_string() {
                    log::info!("value is changed.. {new_value}");
                    <Self as Component>::update(self, Msg::ChangeValue(new_value));
                }
            }
            "syntax" => {
                if let Some(new_value) = new_value[0].as_string() {
                    log::info!("syntax token is changed: {new_value}");
                    <Self as Component>::update(self, Msg::ChangeSyntax(new_value));
                }
            }
            "theme" => {
                if let Some(new_value) = new_value[0].as_string() {
                    log::info!("theme is changed: {new_value}");
                    <Self as Component>::update(self, Msg::ChangeTheme(new_value));
                }
            }
            _ => (),
        }
    }

    fn connected_callback(&mut self) {}
    fn disconnected_callback(&mut self) {}
    fn adopted_callback(&mut self) {}
    fn child_container(&self) -> Option<DomNode> {
        None
    }
}



pub mod attributes {
    use sauron::html::attributes::attr;
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
) -> Node<MSG>
where
    MSG: 'static,
{

    if !children.into_iter().collect::<Vec<_>>().is_empty() {
        log::warn!("ultron editor ignore the passed children nodes");
    }
    stateful_component(WebEditor::default(), attrs, [])
}
