use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveEmptyAttrs;

impl Plugin for RemoveEmptyAttrs {
    fn apply(&self, doc: &mut Document) {
        remove_empty_attrs_in_nodes(&mut doc.root);
    }
}

fn remove_empty_attrs_in_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            // Retain only non-empty attributes
            // Special case: some attributes might be meaningful if empty?
            // Generally in SVG, empty attributes are ignored or valid but useless for display.
            // We'll remove them.
            elem.attributes.retain(|_, v| !v.is_empty());

            remove_empty_attrs_in_nodes(&mut elem.children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_empty_attrs() {
        let input = "<svg><rect width=\"100\" height=\"\" class=\"\"/></svg>";
        let expected = "<svg><rect width=\"100\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyAttrs.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }
}
