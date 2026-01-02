use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveDimensions;

impl Plugin for RemoveDimensions {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "svg" {
                if elem.attributes.contains_key("viewBox") {
                    elem.attributes.shift_remove("width");
                    elem.attributes.shift_remove("height");
                }
            }
            // Usually only on root svg, but recursively correct for nested SVGs too.
            process_nodes(&mut elem.children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_dimensions() {
        let input = "<svg width=\"100\" height=\"100\" viewBox=\"0 0 100 100\"><rect/></svg>";
        let expected = "<svg viewBox=\"0 0 100 100\"><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveDimensions.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_dimensions_if_no_viewbox() {
        let input = "<svg width=\"100\" height=\"100\"><rect/></svg>";
        let mut doc = parser::parse(input).unwrap();
        RemoveDimensions.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
