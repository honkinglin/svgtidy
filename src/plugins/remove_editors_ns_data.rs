use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::collections::HashSet;

pub struct RemoveEditorsNSData;

impl Plugin for RemoveEditorsNSData {
    fn apply(&self, doc: &mut Document) {
        // Known editor namespaces
        let editor_namespaces = [
            "http://www.inkscape.org/namespaces/inkscape",
            "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd",
            "http://ns.adobe.com/AdobeIllustrator/10.0/",
            "http://ns.adobe.com/SaveForWeb/1.0/",
        ];

        // 1. Find prefixes bound to these namespaces (usually on root, but could be anywhere.
        //    For simplicity, we verify root attributes for now as most editors put them there).
        //    A robust implementation would track scope, but global removal is usually safe for optimization.
        let mut prefixes_to_remove = HashSet::new();

        // Helper to scan for xmlns definitions
        fn scan_prefixes(nodes: &[Node], prefixes: &mut HashSet<String>, ns_list: &[&str]) {
            for node in nodes {
                if let Node::Element(elem) = node {
                    for (k, v) in &elem.attributes {
                        if k.starts_with("xmlns:") {
                            let prefix = &k[6..];
                            if ns_list.contains(&v.as_str()) {
                                prefixes.insert(prefix.to_string());
                            }
                        }
                    }
                    scan_prefixes(&elem.children, prefixes, ns_list);
                }
            }
        }

        scan_prefixes(&doc.root, &mut prefixes_to_remove, &editor_namespaces);

        // 2. Remove matching xmlns attributes and Namespaced elements/attributes
        remove_ns_data(&mut doc.root, &prefixes_to_remove, &editor_namespaces);
    }
}

fn remove_ns_data(nodes: &mut Vec<Node>, prefixes: &HashSet<String>, ns_list: &[&str]) {
    // Remove elements with matching prefix
    nodes.retain(|node| {
        if let Node::Element(elem) = node {
            if let Some((prefix, _)) = elem.name.split_once(':') {
                if prefixes.contains(prefix) {
                    return false;
                }
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            // Remove attributes
            elem.attributes.retain(|k, v| {
                // Remove xmlns:prefix="EDITOR_URI"
                if k.starts_with("xmlns:") {
                    let prefix = &k[6..];
                    if prefixes.contains(prefix) && ns_list.contains(&v.as_str()) {
                        return false;
                    }
                }

                // Remove prefix:attr
                if let Some((prefix, _)) = k.split_once(':') {
                    if prefixes.contains(prefix) {
                        return false;
                    }
                }

                true
            });

            // Recurse
            remove_ns_data(&mut elem.children, prefixes, ns_list);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_editors_ns_data() {
        let input = r#"<svg xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" inkscape:version="1.0" width="100"><sodipodi:namedview id="base"/><rect width="10" height="10" inkscape:label="rect1"/></svg>"#;

        // Should remove xmlns declarations, sodipodi element, inkscape attributes
        let expected = "<svg width=\"100\"><rect width=\"10\" height=\"10\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        // pre-clean to avoid whitespace issues in comparison (optional, but helps match expected exactly if test input has newlines)
        // But here we rely on printer behavior. Printer doesn't format text nodes, which might contain whitespace from input.
        // The input has whitespace text nodes (indentation).
        // Let's assume a cleaner input or rely on the logic ignoring text nodes only if we don't care about them.
        // Actually, our parser keeps text nodes. So the output will contain "        " text nodes unless we remove them.
        // To verify LOGIC, better use input without indentation or handle text.

        RemoveEditorsNSData.apply(&mut doc);

        // For accurate string comparison, checking logic specifically
        let output = printer::print(&doc);
        assert_eq!(output, expected);
    }
}
