use std::collections::HashMap;

use mmot_core::{parser, pipeline, renderer};

#[test]
#[ignore]
fn generate_golden_minimal() {
    let json = std::fs::read_to_string("../../tests/fixtures/valid/minimal.mmot.json").unwrap();
    let scene = parser::parse(&json).unwrap();
    let no_fonts = HashMap::new();
    let frame_scene = pipeline::evaluate_scene(&scene, 0, &no_fonts).unwrap();
    let w = frame_scene.width;
    let h = frame_scene.height;
    let rgba = renderer::render(&frame_scene).unwrap();
    let img = image::RgbaImage::from_raw(w, h, rgba).expect("invalid dimensions");
    std::fs::create_dir_all("tests/golden/minimal").unwrap();
    img.save("tests/golden/minimal/frame-000.png").unwrap();
    println!("Golden image saved to tests/golden/minimal/frame-000.png");
}

#[test]
#[ignore]
fn generate_golden_gradient() {
    let json =
        std::fs::read_to_string("../../tests/fixtures/valid/gradient.mmot.json").unwrap();
    let scene = parser::parse(&json).unwrap();
    let no_fonts = HashMap::new();
    let frame_scene = pipeline::evaluate_scene(&scene, 0, &no_fonts).unwrap();
    let w = frame_scene.width;
    let h = frame_scene.height;
    let rgba = renderer::render(&frame_scene).unwrap();
    let img = image::RgbaImage::from_raw(w, h, rgba).expect("invalid dimensions");
    std::fs::create_dir_all("tests/golden/gradient").unwrap();
    img.save("tests/golden/gradient/frame-000.png").unwrap();
    println!("Golden image saved to tests/golden/gradient/frame-000.png");
}
