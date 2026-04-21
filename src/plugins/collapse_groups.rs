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
        if let Node::Element(mut elem) = node {
            if can_move_group_id_to_child(&elem) {
                let id = elem.attributes.shift_remove("id").unwrap();
                let child = elem.children.pop().unwrap();

                if let Node::Element(mut child_elem) = child {
                    child_elem.attributes.insert("id".to_string(), id);
                    new_nodes.push(Node::Element(child_elem));
                }
            } else if can_unwrap_group(&elem) {
                new_nodes.extend(elem.children);
            } else {
                new_nodes.push(Node::Element(elem));
            }
        } else {
            new_nodes.push(node);
        }
    }

    *nodes = new_nodes;
}

fn can_unwrap_group(elem: &crate::tree::Element) -> bool {
    elem.name == "g" && elem.attributes.is_empty() && !has_group_semantic_children(elem)
}

fn can_move_group_id_to_child(elem: &crate::tree::Element) -> bool {
    if elem.name != "g" || has_group_semantic_children(elem) {
        return false;
    }

    if elem.attributes.len() != 1 || !elem.attributes.contains_key("id") {
        return false;
    }

    if elem.children.len() != 1 {
        return false;
    }

    matches!(
        elem.children.first(),
        Some(Node::Element(child_elem)) if !child_elem.attributes.contains_key("id")
    )
}

fn has_group_semantic_children(elem: &crate::tree::Element) -> bool {
    elem.children.iter().any(|child| {
        matches!(
            child,
            Node::Element(child_elem)
                if matches!(
                    child_elem.name.as_str(),
                    "animate"
                        | "animateColor"
                        | "animateMotion"
                        | "animateTransform"
                        | "desc"
                        | "set"
                        | "title"
                )
        )
    })
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
        let expected = "<svg><rect id=\"g1\"/></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_group_with_id_when_multiple_children() {
        let input = "<svg><g id=\"g1\"><rect/><circle/></g></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_keep_group_with_id_when_child_already_has_id() {
        let input = "<svg><g id=\"g1\"><rect id=\"child\"/></g></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_multiple_children() {
        let input = "<svg><g><rect/><circle/></g></svg>";
        let expected = "<svg><rect/><circle/></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_group_with_title_child() {
        let input = "<svg><g><title>Label</title><rect/></g></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_keep_group_with_animation_child() {
        let input = "<svg><g><animateTransform attributeName=\"transform\" type=\"scale\" from=\"1\" to=\"2\" dur=\"1s\"/><rect/></g></svg>";
        let mut doc = parser::parse(input).unwrap();
        CollapseGroups.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
