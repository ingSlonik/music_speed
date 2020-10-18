use wasm_bindgen::prelude::*;

use music_speed;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn analyse(conf: music_speed::Configuration) {
    music_speed::analyse(conf)
}
