use std::collections::HashMap;
use std::path::Path;

use mmot_core::{parser, pipeline, renderer};

fn render_frame_rgba(fixture: &str, frame: u64) -> Vec<u8> {
    let json =
        std::fs::read_to_string(Path::new("../../tests/fixtures/valid").join(fixture)).unwrap();
    let scene = parser::parse(&json).unwrap();
    let no_fonts = HashMap::new();
    let frame_scene = pipeline::evaluate_scene(&scene, frame, &no_fonts).unwrap();
    renderer::render(&frame_scene).unwrap()
}

fn load_reference_png(name: &str, frame: u64) -> Vec<u8> {
    let path = format!("tests/golden/{name}/frame-{frame:03}.png");
    let img = image::open(&path)
        .unwrap_or_else(|_| panic!("reference image not found: {path}"))
        .into_rgba8();
    img.into_raw()
}

#[test]
fn golden_minimal_frame_0() {
    let rendered = render_frame_rgba("minimal.mmot.json", 0);
    let reference = load_reference_png("minimal", 0);
    assert_eq!(rendered.len(), reference.len(), "frame size mismatch");
    assert_eq!(rendered, reference, "pixel mismatch — renderer output changed");
}
