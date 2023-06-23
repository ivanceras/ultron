use ultron_web::sauron::*;

#[wasm_bindgen(start)]
pub fn main() {
    ultron_web::WebEditorCustomElement::register();
}
