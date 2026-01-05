use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct MoveGroupAttrsToElems;

impl Plugin for MoveGroupAttrsToElems {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    // We need to iterate and modify.
    // If we modify a group, we might need to process its children again?
    // Let's do a recursive traversal.

    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "g" {
                // Try to move attributes to children
                if !elem.children.is_empty() {
                    let mut cached_transform = elem.attributes.get("transform").cloned();

                    // inheritable attributes
                    // If we move them, we remove them from group.
                    // But we can only move if ALL children can accept them?
                    // Or we just push them down?
                    // SVGO logic: if group has nothing else, we can push down.
                    // This is complex. Let's strictly implement Transform propagation for now as it solves the "unwrappable group" issue.

                    if let Some(ref root_transform) = cached_transform {
                        for child in &mut elem.children {
                            if let Node::Element(child_elem) = child {
                                // Prepend group transform to child transform
                                if let Some(child_t) = child_elem.attributes.get_mut("transform") {
                                    *child_t = format!("{} {}", root_transform, child_t);
                                } else {
                                    child_elem
                                        .attributes
                                        .insert("transform".to_string(), root_transform.clone());
                                }
                            }
                        }
                        // Remove from group
                        elem.attributes.shift_remove("transform");
                    }
                }
            }

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
    fn test_move_transform() {
        let input = "<svg><g transform=\"scale(2)\"><rect width=\"10\"/><circle/></g></svg>";
        let expected = "<svg><g><rect width=\"10\" transform=\"scale(2)\"/><circle transform=\"scale(2)\"/></g></svg>";
        // Note: CollapseGroups would later remove the empty <g>

        let mut doc = parser::parse(input).unwrap();
        MoveGroupAttrsToElems.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_concat_transform() {
        let input = "<svg><g transform=\"translate(10)\"><rect transform=\"scale(2)\"/></g></svg>";
        let expected = "<svg><g><rect transform=\"translate(10) scale(2)\"/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        MoveGroupAttrsToElems.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }
}
