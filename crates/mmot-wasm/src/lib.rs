//! WASM browser renderer for Mercury-Motion.
//!
//! Compiles mmot-core's evaluation pipeline + a tiny-skia renderer to WebAssembly.
//! Provides `validate`, `get_scene_info`, and `render_frame` functions callable
//! from JavaScript via wasm-bindgen.
//!
//! Build with: `wasm-pack build crates/mmot-wasm --target web`

mod renderer;

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

/// Validate a `.mmot.json` string.
///
/// Returns a JSON string: `{"valid": true}` or `{"valid": false, "error": "..."}`.
#[wasm_bindgen]
pub fn validate(json: &str) -> String {
    match mmot_core::parser::parse(json) {
        Ok(_) => serde_json::json!({"valid": true}).to_string(),
        Err(e) => serde_json::json!({"valid": false, "error": e.to_string()}).to_string(),
    }
}

/// Get scene metadata without rendering.
///
/// Returns a JSON string with width, height, fps, duration, etc.
#[wasm_bindgen]
pub fn get_scene_info(json: &str) -> String {
    match mmot_core::pipeline::get_scene_info(json) {
        Ok(info) => serde_json::json!({
            "width": info.width,
            "height": info.height,
            "fps": info.fps,
            "duration_frames": info.duration_frames,
            "duration_secs": info.duration_secs,
            "composition_count": info.composition_count,
            "root_layer_count": info.root_layer_count,
        })
        .to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Render a single frame as RGBA bytes.
///
/// Takes a `.mmot.json` string and a frame number.
/// Returns raw RGBA pixel data (width * height * 4 bytes).
#[wasm_bindgen]
pub fn render_frame(json: &str, frame: u64) -> Result<Vec<u8>, JsValue> {
    let scene = mmot_core::parser::parse(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let font_cache: HashMap<String, Vec<u8>> = HashMap::new();

    let frame_scene = mmot_core::pipeline::evaluate_scene(&scene, frame, &font_cache)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    renderer::render_frame_tiny_skia(&frame_scene)
        .map_err(|e| JsValue::from_str(&e))
}

/// Get the dimensions of a scene (width, height) as a two-element array.
///
/// Useful for setting up the canvas size before rendering.
#[wasm_bindgen]
pub fn get_dimensions(json: &str) -> Result<Vec<u32>, JsValue> {
    let scene = mmot_core::parser::parse(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(vec![scene.meta.width, scene.meta.height])
}
