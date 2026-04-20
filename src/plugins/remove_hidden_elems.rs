use crate::plugins::collections::{find_used_ids, node_has_used_id};
use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::collections::HashSet;

pub struct RemoveHiddenElems;

impl Plugin for RemoveHiddenElems {
    fn apply(&self, doc: &mut Document) {
        let mut used_ids = HashSet::new();
        for node in &doc.root {
            find_used_ids(node, &mut used_ids);
        }
        remove_hidden_elems_from_nodes(&mut doc.root, &used_ids);
    }
}

fn remove_hidden_elems_from_nodes(nodes: &mut Vec<Node>, used_ids: &HashSet<String>) {
    nodes.retain(|node| !should_remove_node(node, used_ids));

    for node in nodes {
        if let Node::Element(elem) = node {
            remove_hidden_elems_from_nodes(&mut elem.children, used_ids);
        }
    }
}

fn should_remove_node(node: &Node, used_ids: &HashSet<String>) -> bool {
    let Node::Element(elem) = node else {
        return false;
    };

    if node_has_used_id(node, used_ids) {
        return false;
    }

    if elem
        .attributes
        .get("display")
        .is_some_and(|value| value == "none")
    {
        return true;
    }

    if elem.name == "circle" && elem.attributes.get("r").is_some_and(|value| is_zero(value)) {
        return true;
    }

    if elem.name == "rect"
        && (elem
            .attributes
            .get("width")
            .is_some_and(|value| is_zero(value))
            || elem
                .attributes
                .get("height")
                .is_some_and(|value| is_zero(value)))
    {
        return true;
    }

    false
}

fn is_zero(value: &str) -> bool {
    matches!(value, "0" | "0px" | "0pt" | "0em")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_hidden() {
        let input = "<svg><rect display=\"none\"/><g>visible</g></svg>";
        let expected = "<svg><g>visible</g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveHiddenElems.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_keep_opacity_zero() {
        let input = "<svg><circle opacity=\"0\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveHiddenElems.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
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

    #[test]
    fn test_keep_referenced_hidden_node() {
        let input =
            "<svg><defs><g id=\"icon\" display=\"none\"><path d=\"M0 0L1 1\"/></g></defs><use href=\"#icon\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveHiddenElems.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_keep_referenced_zero_size_shape() {
        let input = "<svg><defs><rect id=\"shape\" width=\"0\" height=\"10\"/></defs><use href=\"#shape\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveHiddenElems.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }
}
