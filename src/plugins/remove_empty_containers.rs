use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveEmptyContainers;

impl Plugin for RemoveEmptyContainers {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes.iter_mut() {
        if let Node::Element(elem) = node {
            process_nodes(&mut elem.children);
        }
    }

    nodes.retain(|node| {
        let Node::Element(elem) = node else {
            return true;
        };

        if !is_empty_container(elem.name.as_str()) {
            return true;
        }

        !elem.children.is_empty()
    });
}

fn is_empty_container(name: &str) -> bool {
    matches!(
        name,
        "defs" | "g" | "marker" | "mask" | "missing-glyph" | "pattern" | "switch" | "symbol"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_empty_g() {
        let input = "<svg><g/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyContainers.apply(&mut doc);

        assert_eq!(printer::print(&doc), "<svg/>");
    }

    #[test]
    fn test_remove_empty_defs() {
        let input = "<svg><defs/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyContainers.apply(&mut doc);

        assert_eq!(printer::print(&doc), "<svg/>");
    }

    #[test]
    fn test_remove_nested_empty_groups() {
        let input = "<svg><g><g/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyContainers.apply(&mut doc);

        assert_eq!(printer::print(&doc), "<svg/>");
    }

    #[test]
    fn test_keep_filled() {
        let input = "<svg><g><rect/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyContainers.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }
}
