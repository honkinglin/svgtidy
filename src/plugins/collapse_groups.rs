use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct CollapseGroups;

impl Plugin for CollapseGroups {
    fn apply(&self, doc: &mut Document) {
        // Pass "svg" as logical parent of roots
        collapse_groups_in_nodes(&mut doc.root, "svg");
    }
}

fn collapse_groups_in_nodes(nodes: &mut Vec<Node>, parent_name: &str) {
    // 1. Recurse (bottom-up processing)
    for node in nodes.iter_mut() {
        if let Node::Element(elem) = node {
            collapse_groups_in_nodes(&mut elem.children, &elem.name);
        }
    }

    // 2. Collapse
    let parent_prevents_unwrap = match parent_name {
        "switch" | "foreignObject" => true,
        _ => false,
    };

    if parent_prevents_unwrap {
        return;
    }

    let mut new_nodes = Vec::with_capacity(nodes.len());

    for node in nodes.drain(..) {
        if let Node::Element(elem) = node {
            if elem.name == "g" && elem.attributes.is_empty() {
                // Unwrap: Move children to parent
                new_nodes.extend(elem.children);
            } else {
                // Keep
                new_nodes.push(Node::Element(elem));
            }
        } else {
            new_nodes.push(node);
        }
    }

    *nodes = new_nodes;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_collapse_group() {
        let input = "<svg><g><rect/></g></svg>";
        let expected = "<svg><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_nested_collapse() {
        let input = "<svg><g><g><rect/></g></g></svg>";
        let expected = "<svg><rect/></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_attr_group() {
        let input = "<svg><g id=\"g1\"><rect/></g></svg>";
        let expected = "<svg><g id=\"g1\"><rect/></g></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_multiple_children() {
        let input = "<svg><g><rect/><circle/></g></svg>";
        let expected = "<svg><rect/><circle/></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }
}
