use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/index.js")]
extern "C" {
    #[wasm_bindgen]
    pub fn editor_init() -> JsValue;

    #[wasm_bindgen]
    pub fn editor_text(_: &JsValue) -> String;
}

#[derive(Debug, Default)]
pub struct Editor {
    object: Option<JsValue>,
}

impl Editor {
    pub fn init(&mut self) {
        if self.object.is_none() {
            self.object = Some(editor_init());
        }
    }

    pub fn text(&self) -> Option<String> {
        self.object.as_ref().map(|object| editor_text(object))
    }
}
