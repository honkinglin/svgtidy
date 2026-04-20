use crate::plugins::collections::{find_used_ids, node_has_used_id};
use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::collections::HashSet;

pub struct RemoveUselessDefs;

impl Plugin for RemoveUselessDefs {
    fn apply(&self, doc: &mut Document) {
        let mut used_ids = HashSet::new();
        for node in &doc.root {
            find_used_ids(node, &mut used_ids);
        }

        remove_useless_defs_in_nodes(&mut doc.root, &used_ids);
    }
}

fn remove_useless_defs_in_nodes(nodes: &mut Vec<Node>, used_ids: &HashSet<String>) {
    // 1. Recurse first (to clean nested defs)
    for node in nodes.iter_mut() {
        if let Node::Element(elem) = node {
            remove_useless_defs_in_nodes(&mut elem.children, used_ids);

            // If there is ANY defs element, filter its children now (mutable access)
            if elem.name == "defs" {
                elem.children.retain(|child| {
                    matches!(child, Node::Element(_)) && node_has_used_id(child, used_ids)
                });
            }
        }
    }

    // 2. Remove empty defs (retain)
    nodes.retain(|node| {
        if let Node::Element(elem) = node {
            if elem.name == "defs" && elem.children.is_empty() {
                return false;
            }
        }
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_useless_defs() {
        let input =
            "<svg><defs><rect id=\"unused\"/><rect id=\"used\"/></defs><use href=\"#used\"/></svg>";
        let expected = "<svg><defs><rect id=\"used\"/></defs><use href=\"#used\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessDefs.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_remove_empty_defs() {
        let input = "<svg><defs><rect id=\"unused\"/></defs></svg>";
        let expected = "<svg/>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessDefs.apply(&mut doc);
        let output = printer::print(&doc);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_keep_defs_container_when_nested_child_is_used() {
        let input = "<svg><defs><g><path id=\"used\"/></g></defs><use href=\"#used\"/></svg>";
        let expected = "<svg><defs><g><path id=\"used\"/></g></defs><use href=\"#used\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessDefs.apply(&mut doc);
        let output = printer::print(&doc);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_keep_defs_when_aria_uses_nested_id() {
        let input =
            "<svg><defs><g><title id=\"title\">T</title></g></defs><rect aria-labelledby=\"title\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessDefs.apply(&mut doc);
        let output = printer::print(&doc);
        assert!(output.contains("<defs><g><title id=\"title\">T</title></g></defs>"));
    }
}
