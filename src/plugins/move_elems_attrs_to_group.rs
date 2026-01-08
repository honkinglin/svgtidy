use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct MoveElemsAttrsToGroup;

impl Plugin for MoveElemsAttrsToGroup {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "g" && !elem.children.is_empty() {
                // Check for common attributes in all Element children
                // Ignore text/comments for this check?
                // Usually applies if all *Element* children have it.

                let mut common_attrs = Vec::new();

                // Get first child attributes
                let mut first_child_attrs = None;
                let mut element_children_count = 0;

                for child in &elem.children {
                    if let Node::Element(child_elem) = child {
                        element_children_count += 1;
                        if first_child_attrs.is_none() {
                            first_child_attrs = Some(child_elem.attributes.clone());
                        }
                    }
                }

                if element_children_count > 0 {
                    if let Some(candidates) = first_child_attrs {
                        for (k, v) in candidates {
                            if k == "transform" || k == "id" || k == "d" {
                                continue;
                            }

                            // Only move inheritable attributes
                            let inheritable = matches!(
                                k.as_str(),
                                "clip-rule"
                                    | "color"
                                    | "color-interpolation"
                                    | "color-interpolation-filters"
                                    | "color-profile"
                                    | "color-rendering"
                                    | "cursor"
                                    | "direction"
                                    | "fill"
                                    | "fill-opacity"
                                    | "fill-rule"
                                    | "font"
                                    | "font-family"
                                    | "font-size"
                                    | "font-size-adjust"
                                    | "font-stretch"
                                    | "font-style"
                                    | "font-variant"
                                    | "font-weight"
                                    | "glyph-orientation-horizontal"
                                    | "glyph-orientation-vertical"
                                    | "image-rendering"
                                    | "kerning"
                                    | "letter-spacing"
                                    | "marker"
                                    | "marker-end"
                                    | "marker-mid"
                                    | "marker-start"
                                    | "pointer-events"
                                    | "shape-rendering"
                                    | "stroke"
                                    | "stroke-dasharray"
                                    | "stroke-dashoffset"
                                    | "stroke-linecap"
                                    | "stroke-linejoin"
                                    | "stroke-miterlimit"
                                    | "stroke-opacity"
                                    | "stroke-width"
                                    | "text-anchor"
                                    | "text-rendering"
                                    | "visibility"
                                    | "white-space"
                                    | "word-spacing"
                                    | "writing-mode"
                            );

                            if !inheritable {
                                continue;
                            }

                            // Check if all other children have this
                            let mut all_match = true;
                            for child in &elem.children {
                                if let Node::Element(child_elem) = child {
                                    if child_elem.attributes.get(&k) != Some(&v) {
                                        all_match = false;
                                        break;
                                    }
                                }
                            }

                            if all_match {
                                common_attrs.push((k, v));
                            }
                        }
                    }
                }

                // Move common attrs to group
                for (k, v) in common_attrs {
                    elem.attributes.insert(k.clone(), v);
                    // Remove from children
                    for child in &mut elem.children {
                        if let Node::Element(child_elem) = child {
                            child_elem.attributes.shift_remove(&k);
                        }
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
    fn test_move_up_fill() {
        let input = "<svg><g><rect fill=\"red\"/><circle fill=\"red\"/></g></svg>";
        let expected = "<svg><g fill=\"red\"><rect/><circle/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        MoveElemsAttrsToGroup.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_no_move_mixed() {
        let input = "<svg><g><rect fill=\"red\"/><circle fill=\"blue\"/></g></svg>";
        let mut doc = parser::parse(input).unwrap();
        MoveElemsAttrsToGroup.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
