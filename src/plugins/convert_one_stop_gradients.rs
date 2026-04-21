use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::collections::HashMap;

pub struct ConvertOneStopGradients;

impl Plugin for ConvertOneStopGradients {
    fn apply(&self, doc: &mut Document) {
        // Phase 1: Find 1-stop gradients
        let gradients = find_one_stop_gradients(&doc.root);

        if !gradients.is_empty() {
            // Phase 2: Replace usages
            replace_usages(&mut doc.root, &gradients);
        }
    }
}

fn find_one_stop_gradients(nodes: &[Node]) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "linearGradient" || elem.name == "radialGradient" {
                if let Some(id) = elem.attributes.get("id") {
                    let mut stop_color = None;
                    let mut stop_count = 0;
                    let mut unsupported_stop = false;

                    for child in &elem.children {
                        if let Node::Element(child_elem) = child {
                            if child_elem.name == "stop" {
                                stop_count += 1;
                                if stop_count == 1 {
                                    if let Some(color) = extract_stop_color(child_elem) {
                                        stop_color = Some(color);
                                    } else {
                                        unsupported_stop = true;
                                    }
                                }
                            }
                        }
                    }

                    if stop_count == 1 && !unsupported_stop {
                        if let Some(c) = stop_color {
                            map.insert(id.clone(), c);
                        }
                    }
                }
            }
            let sub = find_one_stop_gradients(&elem.children);
            map.extend(sub);
        }
    }
    map
}

fn extract_stop_color(elem: &crate::tree::Element) -> Option<String> {
    if elem
        .attributes
        .get("stop-opacity")
        .is_some_and(|value| value.trim() != "1")
    {
        return None;
    }

    if let Some(style) = elem.attributes.get("style") {
        let declarations = parse_style(style)?;
        if declarations
            .iter()
            .any(|(key, value)| key == "stop-opacity" && value.trim() != "1")
        {
            return None;
        }
        if let Some((_, value)) = declarations.iter().find(|(key, _)| key == "stop-color") {
            return Some(value.clone());
        }
    }

    elem.attributes.get("stop-color").cloned()
}

fn parse_style(style: &str) -> Option<Vec<(String, String)>> {
    let mut declarations = Vec::new();

    for raw in style.split(';') {
        let decl = raw.trim();
        if decl.is_empty() {
            continue;
        }

        let (key, value) = decl.split_once(':')?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return None;
        }

        declarations.push((key.to_string(), value.to_string()));
    }

    Some(declarations)
}

fn replace_usages(nodes: &mut Vec<Node>, map: &HashMap<String, String>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            // Check fill and stroke
            for attr in ["fill", "stroke"] {
                if let Some(val) = elem.attributes.get_mut(attr) {
                    if val.starts_with("url(#") && val.ends_with(")") {
                        let id = &val[5..val.len() - 1];
                        if let Some(color) = map.get(id) {
                            *val = color.clone();
                        }
                    }
                }
            }
            replace_usages(&mut elem.children, map);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_convert_gradient() {
        let input = r#"<svg>
        <defs>
            <linearGradient id="g1"><stop stop-color="red"/></linearGradient>
        </defs>
        <rect fill="url(#g1)"/>
        </svg>"#;

        let _expected = r#"<svg>
        <defs>
            <linearGradient id="g1"><stop stop-color="red"/></linearGradient>
        </defs>
        <rect fill="red"/>
        </svg>"#;
        // Note: removeUselessDefs or cleanupIds would later remove the unused gradient def.

        let mut doc = parser::parse(input).unwrap();
        ConvertOneStopGradients.apply(&mut doc);
        // We use whitespace insensitive comparison or printer normalization
        let out = printer::print(&doc);
        // Simple check
        assert!(out.contains("fill=\"red\""));
        assert!(!out.contains("fill=\"url(#g1)\""));
    }

    #[test]
    fn test_convert_gradient_stop_color_from_style() {
        let input = r#"<svg><defs><linearGradient id="g1"><stop style="stop-color:#00f;stop-opacity:1"/></linearGradient></defs><rect fill="url(#g1)"/></svg>"#;

        let mut doc = parser::parse(input).unwrap();
        ConvertOneStopGradients.apply(&mut doc);
        let out = printer::print(&doc);

        assert!(out.contains("fill=\"#00f\""));
        assert!(!out.contains("fill=\"url(#g1)\""));
    }

    #[test]
    fn test_keep_gradient_when_stop_opacity_is_present() {
        let input = r#"<svg><defs><linearGradient id="g1"><stop stop-color="red" stop-opacity=".5"/></linearGradient></defs><rect fill="url(#g1)"/></svg>"#;

        let mut doc = parser::parse(input).unwrap();
        ConvertOneStopGradients.apply(&mut doc);
        let out = printer::print(&doc);

        assert!(out.contains("fill=\"url(#g1)\""));
    }

    #[test]
    fn test_keep_zero_stop_gradient() {
        let input =
            r#"<svg><defs><linearGradient id="g1"/></defs><rect fill="url(#g1)"/></svg>"#;

        let mut doc = parser::parse(input).unwrap();
        ConvertOneStopGradients.apply(&mut doc);
        let out = printer::print(&doc);

        assert!(out.contains("fill=\"url(#g1)\""));
    }
}
