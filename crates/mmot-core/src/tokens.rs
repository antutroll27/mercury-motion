use std::collections::HashMap;

use serde_json::Value;

/// Resolve `$token.name` references in a JSON string using the tokens map.
///
/// Strategy: parse the JSON to extract tokens, then do string replacement
/// on the original JSON for all `$token.name` references, then re-parse.
pub fn resolve_tokens(json: &str) -> crate::error::Result<String> {
    // First, parse just enough to extract the tokens map
    let raw: Value = serde_json::from_str(json).map_err(|e| crate::error::MmotError::Parse {
        message: e.to_string(),
        pointer: String::new(),
    })?;

    let tokens = match raw.get("tokens") {
        Some(Value::Object(map)) => {
            let mut tokens = HashMap::new();
            for (key, value) in map {
                tokens.insert(key.clone(), value.clone());
            }
            tokens
        }
        _ => return Ok(json.to_string()), // No tokens, return as-is
    };

    if tokens.is_empty() {
        return Ok(json.to_string());
    }

    // Replace all $token.name references in the JSON string.
    // Only token values that DON'T contain $ references are substituted,
    // preventing recursive/chained token expansion (single pass, no cascading).
    let mut result = json.to_string();

    // Sort tokens by key length (longest first) to avoid partial matches
    let mut sorted_keys: Vec<&String> = tokens.keys().collect();
    sorted_keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

    for key in sorted_keys {
        let value = &tokens[key];
        let token_ref = format!("${}", key);

        // Replace "$token.name" (with quotes) with the token value
        // For string values: "$brand.primary" -> "#c1121f"  (keep quotes)
        // For number values: "$duration.fast" -> 8  (remove quotes)
        // For object values: "$easing.bounce" -> {"type":"spring",...}  (remove quotes)

        match value {
            Value::String(s) => {
                // String token: replace the token ref keeping surrounding quotes
                result = result.replace(&format!("\"{}\"", token_ref), &format!("\"{}\"", s));
                // Also replace bare (unquoted) references
                result = result.replace(&token_ref, s);
            }
            Value::Number(n) => {
                // Number token: replace quoted ref with bare number
                result = result.replace(&format!("\"{}\"", token_ref), &n.to_string());
                result = result.replace(&token_ref, &n.to_string());
            }
            _ => {
                // Object/array/bool token: replace quoted ref with JSON
                let json_str = value.to_string();
                result = result.replace(&format!("\"{}\"", token_ref), &json_str);
                result = result.replace(&token_ref, &json_str);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_string_token() {
        let json = r##"{
            "tokens": { "brand.primary": "#c1121f" },
            "color": "$brand.primary"
        }"##;
        let resolved = resolve_tokens(json).expect("should resolve");
        let v: Value = serde_json::from_str(&resolved).expect("valid json");
        assert_eq!(v["color"], "#c1121f");
    }

    #[test]
    fn resolve_number_token() {
        let json = r#"{
            "tokens": { "duration.fast": 8 },
            "frames": "$duration.fast"
        }"#;
        let resolved = resolve_tokens(json).expect("should resolve");
        let v: Value = serde_json::from_str(&resolved).expect("valid json");
        assert_eq!(v["frames"], 8);
    }

    #[test]
    fn resolve_object_token() {
        let json = r#"{
            "tokens": {
                "easing.bounce": { "type": "spring", "stiffness": 170, "damping": 20, "mass": 1.0 }
            },
            "easing": "$easing.bounce"
        }"#;
        let resolved = resolve_tokens(json).expect("should resolve");
        let v: Value = serde_json::from_str(&resolved).expect("valid json");
        assert_eq!(v["easing"]["type"], "spring");
        assert_eq!(v["easing"]["stiffness"], 170);
    }

    #[test]
    fn resolve_no_tokens() {
        let json = r#"{"version": "1.0", "name": "Test"}"#;
        let resolved = resolve_tokens(json).expect("should resolve");
        assert_eq!(resolved, json);
    }

    #[test]
    fn resolve_nested_reference() {
        let json = r##"{
            "tokens": { "brand.primary": "#c1121f" },
            "layers": [{
                "transform": {
                    "color": "$brand.primary"
                }
            }]
        }"##;
        let resolved = resolve_tokens(json).expect("should resolve");
        let v: Value = serde_json::from_str(&resolved).expect("valid json");
        assert_eq!(v["layers"][0]["transform"]["color"], "#c1121f");
    }

    #[test]
    fn resolve_multiple_tokens() {
        let json = r##"{
            "tokens": {
                "brand.primary": "#c1121f",
                "brand.secondary": "#003049",
                "font.heading": "Playfair Display"
            },
            "color1": "$brand.primary",
            "color2": "$brand.secondary",
            "font": "$font.heading"
        }"##;
        let resolved = resolve_tokens(json).expect("should resolve");
        let v: Value = serde_json::from_str(&resolved).expect("valid json");
        assert_eq!(v["color1"], "#c1121f");
        assert_eq!(v["color2"], "#003049");
        assert_eq!(v["font"], "Playfair Display");
    }

    #[test]
    fn resolve_full_scene_with_tokens() {
        let json = r##"{
            "version": "1.0",
            "tokens": {
                "brand.primary": "#c1121f",
                "font.heading": "Playfair Display",
                "duration.fast": 8
            },
            "meta": {
                "name": "TokenTest",
                "width": 1920, "height": 1080,
                "fps": 30, "duration": 90,
                "background": "$brand.primary",
                "root": "main"
            },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "title",
                        "type": "text",
                        "in": 0, "out": "$duration.fast",
                        "text": "Hello",
                        "font": {
                            "family": "$font.heading",
                            "size": 48,
                            "color": "$brand.primary"
                        },
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
        let resolved = resolve_tokens(json).expect("should resolve");
        let v: Value = serde_json::from_str(&resolved).expect("valid json");
        assert_eq!(v["meta"]["background"], "#c1121f");
        assert_eq!(v["compositions"]["main"]["layers"][0]["out"], 8);
        assert_eq!(
            v["compositions"]["main"]["layers"][0]["font"]["family"],
            "Playfair Display"
        );
        assert_eq!(
            v["compositions"]["main"]["layers"][0]["font"]["color"],
            "#c1121f"
        );
    }
}
