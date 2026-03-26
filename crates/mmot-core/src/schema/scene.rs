use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::composition::Compositions;

/// Root scene structure of a `.mmot.json` file.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Scene {
    pub version: String,
    pub meta: Meta,
    #[serde(default)]
    pub props: HashMap<String, PropDef>,
    pub compositions: Compositions,
    #[serde(default)]
    pub assets: Assets,
}

/// Video metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Meta {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    /// Duration in frames.
    pub duration: u64,
    #[serde(default = "default_black")]
    pub background: String,
    pub root: String,
    /// Optional safe zone for multi-format export.
    /// Content inside the safe zone is guaranteed visible in all export profiles.
    #[serde(default)]
    pub safe_zone: Option<SafeZone>,
}

/// A rectangular safe zone within the canvas.
/// Content inside this rectangle is guaranteed to be visible across all
/// export aspect ratios (e.g. YouTube 16:9, Instagram 1:1, TikTok 9:16).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SafeZone {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn default_black() -> String {
    "#000000".into()
}

/// Property definition for template variables.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PropDef {
    #[serde(rename = "type")]
    pub prop_type: PropType,
    pub default: Option<serde_json::Value>,
}

/// Property type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PropType {
    String,
    Color,
    Number,
    Url,
}

/// External assets referenced by the scene.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Assets {
    #[serde(default)]
    pub fonts: Vec<FontAsset>,
}

/// A font asset reference.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FontAsset {
    pub id: String,
    pub src: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::composition::LayerContent;

    #[test]
    fn deserialise_minimal_scene() {
        let json = r##"{
            "version": "1.0",
            "meta": {
                "name": "Test",
                "width": 1920, "height": 1080,
                "fps": 30, "duration": 90,
                "background": "#000000",
                "root": "main"
            },
            "compositions": {
                "main": { "layers": [] }
            }
        }"##;
        let scene: Scene = serde_json::from_str(json).unwrap();
        assert_eq!(scene.meta.width, 1920);
        assert_eq!(scene.meta.fps, 30.0);
        assert_eq!(scene.meta.duration, 90);
        assert!(scene.compositions.contains_key("main"));
    }

    #[test]
    fn deserialise_solid_layer() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":1920,"height":1080,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "bg",
                        "type": "solid",
                        "in": 0, "out": 30,
                        "color": "#ff0000",
                        "transform": {
                            "position": [960.0, 540.0],
                            "scale": [1.0, 1.0],
                            "opacity": 1.0,
                            "rotation": 0.0
                        }
                    }]
                }
            }
        }"##;
        let scene: Scene = serde_json::from_str(json).unwrap();
        let layer = &scene.compositions["main"].layers[0];
        assert_eq!(layer.id, "bg");
        assert!(matches!(layer.content, LayerContent::Solid { .. }));
    }
}
