use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveHiddenElems;

impl Plugin for RemoveHiddenElems {
    fn apply(&self, doc: &mut Document) {
        remove_hidden_elems_from_nodes(&mut doc.root);
    }
}

fn remove_hidden_elems_from_nodes(nodes: &mut Vec<Node>) {
    nodes.retain(|node| {
        if let Node::Element(elem) = node {
            // Check display="none"
            if let Some(val) = elem.attributes.get("display") {
                if val == "none" {
                    return false;
                }
            }
            // Check opacity="0"
            if let Some(val) = elem.attributes.get("opacity") {
                if val == "0" {
                    return false;
                }
            }
            // Check circle with r="0"
            if elem.name == "circle" {
                if let Some(r) = elem.attributes.get("r") {
                    if r == "0" {
                        return false;
                    }
                }
            }
            // Check rect with width="0" or height="0"
            if elem.name == "rect" {
                if let Some(w) = elem.attributes.get("width") {
                    if w == "0" {
                        return false;
                    }
                }
                if let Some(h) = elem.attributes.get("height") {
                    if h == "0" {
                        return false;
                    }
                }
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            remove_hidden_elems_from_nodes(&mut elem.children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_hidden() {
        let input = "<svg><rect display=\"none\"/><circle opacity=\"0\"/><g>visible</g></svg>";
        let expected = "<svg><g>visible</g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveHiddenElems.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_remove_zero_size() {
        let input = "<svg><rect width=\"0\" height=\"10\"/><circle r=\"0\"/></svg>";
        let expected = "<svg/>";

        let mut doc = parser::parse(input).unwrap();
        RemoveHiddenElems.apply(&mut doc);
        let output = printer::print(&doc);
        assert_eq!(output, expected);
    }
}
