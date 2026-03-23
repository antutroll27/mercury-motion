use std::collections::HashMap;

use regex::Regex;

/// Substitute `${varName}` placeholders in a JSON string with provided values.
///
/// Props are provided as key-value string pairs. Any `${key}` in the input
/// is replaced with the corresponding value. Unresolved placeholders are left as-is.
pub fn substitute(json: &str, props: &HashMap<String, String>) -> String {
    if props.is_empty() {
        return json.to_string();
    }

    let re = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("invalid regex");
    re.replace_all(json, |caps: &regex::Captures| {
        let key = &caps[1];
        props.get(key).cloned().unwrap_or_else(|| caps[0].to_string())
    })
    .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitute_replaces_known_props() {
        let mut props = HashMap::new();
        props.insert("title".into(), "Hello World".into());
        props.insert("color".into(), "#FF0000".into());

        let input = r##"{"text": "${title}", "fill": "${color}"}"##;
        let result = substitute(input, &props);
        assert_eq!(result, r##"{"text": "Hello World", "fill": "#FF0000"}"##);
    }

    #[test]
    fn substitute_leaves_unknown_props() {
        let props = HashMap::new();
        let input = r#"{"text": "${unknown}"}"#;
        let result = substitute(input, &props);
        assert_eq!(result, input);
    }

    #[test]
    fn substitute_handles_empty_props() {
        let props = HashMap::new();
        let input = r#"{"text": "no props here"}"#;
        let result = substitute(input, &props);
        assert_eq!(result, input);
    }

    #[test]
    fn substitute_multiple_occurrences() {
        let mut props = HashMap::new();
        props.insert("name".into(), "Mercury".into());

        let input = r#"{"a": "${name}", "b": "${name}"}"#;
        let result = substitute(input, &props);
        assert_eq!(result, r#"{"a": "Mercury", "b": "Mercury"}"#);
    }
}
