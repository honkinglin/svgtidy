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
                    // Count stops
                    let mut stop_color = None;
                    let mut stop_count = 0;

                    for child in &elem.children {
                        if let Node::Element(child_elem) = child {
                            if child_elem.name == "stop" {
                                stop_count += 1;
                                if stop_count == 1 {
                                    // Get color
                                    // stop-color attr or style?
                                    // Simplifying: check stop-color attr first
                                    if let Some(c) = child_elem.attributes.get("stop-color") {
                                        stop_color = Some(c.clone());
                                    }
                                }
                            }
                        }
                    }

                    // SVGO logic: if 0 stops, it's transparent/none?
                    // Let's handle exactly 1 stop for now.
                    // SVGO: "if 0 stops, ???"
                    // Actually if we replace with 1 stop color it works.
                    if stop_count == 1 {
                        if let Some(c) = stop_color {
                            map.insert(id.clone(), c);
                        }
                    } else if stop_count == 0 {
                        // Maybe replace with "none"?
                        map.insert(id.clone(), "none".to_string());
                    }
                }
            }
            // Recurse (gradients nicely nested in defs usually, but could be anywhere)
            let sub = find_one_stop_gradients(&elem.children);
            map.extend(sub);
        }
    }
    map
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

        let expected = r#"<svg>
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
}
