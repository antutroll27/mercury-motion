use std::sync::Mutex;

use tauri::State;

struct EditorState {
    current_scene_json: Mutex<Option<String>>,
}

/// Validate a .mmot.json string and return scene metadata.
#[tauri::command]
fn validate_scene(json: String) -> Result<serde_json::Value, String> {
    match mmot_core::pipeline::get_scene_info(&json) {
        Ok(info) => Ok(serde_json::json!({
            "valid": true,
            "width": info.width,
            "height": info.height,
            "fps": info.fps,
            "duration_frames": info.duration_frames,
            "duration_secs": info.duration_secs,
            "composition_count": info.composition_count,
            "root_layer_count": info.root_layer_count,
        })),
        Err(e) => Ok(serde_json::json!({
            "valid": false,
            "error": e.to_string(),
        })),
    }
}

/// Render a single frame and return RGBA bytes as base64-encoded PNG.
#[tauri::command]
fn render_frame(json: String, frame: u64) -> Result<String, String> {
    let (w, h, rgba) = mmot_core::pipeline::render_single_frame(&json, frame)
        .map_err(|e| e.to_string())?;

    // Convert RGBA to PNG and base64 encode for the frontend
    let img = image::RgbaImage::from_raw(w, h, rgba)
        .ok_or_else(|| "failed to create image from RGBA buffer".to_string())?;

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        w,
        h,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| format!("PNG encode failed: {e}"))?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

/// Render the full scene to a file.
#[tauri::command]
fn render_to_file(
    json: String,
    output_path: String,
    format: String,
    quality: u8,
) -> Result<String, String> {
    let fmt = match format.as_str() {
        "gif" => mmot_core::pipeline::OutputFormat::Gif,
        "webm" => mmot_core::pipeline::OutputFormat::Webm,
        _ => mmot_core::pipeline::OutputFormat::Mp4,
    };

    let opts = mmot_core::pipeline::RenderOptions {
        output_path: output_path.clone().into(),
        format: fmt,
        quality,
        frame_range: None,
        concurrency: None,
        backend: mmot_core::pipeline::RenderBackend::Cpu,
        include_audio: true,
    };

    mmot_core::pipeline::render_scene(&json, opts, None)
        .map_err(|e| e.to_string())?;

    Ok(format!("Rendered to {output_path}"))
}

/// Store the current scene JSON in memory.
#[tauri::command]
fn set_scene(json: String, state: State<EditorState>) -> Result<(), String> {
    let mut scene = state.current_scene_json.lock().map_err(|e| e.to_string())?;
    *scene = Some(json);
    Ok(())
}

/// Get the current scene JSON from memory.
#[tauri::command]
fn get_scene(state: State<EditorState>) -> Result<Option<String>, String> {
    let scene = state.current_scene_json.lock().map_err(|e| e.to_string())?;
    Ok(scene.clone())
}

/// Get the JSON Schema for .mmot.json format.
#[tauri::command]
fn get_schema() -> String {
    use schemars::schema_for;
    let schema = schema_for!(mmot_core::schema::Scene);
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(EditorState {
            current_scene_json: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            validate_scene,
            render_frame,
            render_to_file,
            set_scene,
            get_scene,
            get_schema,
        ])
        .run(tauri::generate_context!())
        .expect("error running Mercury Motion editor");
}
