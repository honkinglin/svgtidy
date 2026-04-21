use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveUselessStrokeAndFill;

impl Plugin for RemoveUselessStrokeAndFill {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    nodes.retain(|node| {
        if let Node::Element(elem) = node {
            let is_shape = matches!(
                elem.name.as_str(),
                "rect" | "circle" | "ellipse" | "line" | "polygon" | "polyline" | "path"
            );
            let stroke_disabled =
                is_explicit_none(elem, "stroke") || has_zero_stroke_width(elem);
            let fill_disabled = is_explicit_none(elem, "fill");

            if is_shape && stroke_disabled && fill_disabled {
                if !elem.attributes.contains_key("id") {
                    return false;
                }
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            cleanup_paint_attrs(elem);
            process_nodes(&mut elem.children);
        }
    }
}

fn cleanup_paint_attrs(elem: &mut crate::tree::Element) {
    if is_explicit_none(elem, "stroke") || has_zero_stroke_width(elem) {
        remove_attrs(
            elem,
            &[
                "stroke",
                "stroke-width",
                "stroke-opacity",
                "stroke-linecap",
                "stroke-linejoin",
                "stroke-miterlimit",
                "stroke-dasharray",
                "stroke-dashoffset",
            ],
        );
    }

    if is_explicit_none(elem, "fill") {
        remove_attrs(elem, &["fill-opacity", "fill-rule"]);
    }
}

fn is_explicit_none(elem: &crate::tree::Element, attr: &str) -> bool {
    elem.attributes.get(attr).is_some_and(|value| value == "none")
}

fn has_zero_stroke_width(elem: &crate::tree::Element) -> bool {
    elem.attributes.get("stroke-width").is_some_and(|value| {
        matches!(value.trim(), "0" | "0px" | "0.0" | "0.0px")
    })
}

fn remove_attrs(elem: &mut crate::tree::Element, attrs: &[&str]) {
    for attr in attrs {
        elem.attributes.shift_remove(*attr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_invisible_rect() {
        let input = "<svg><rect fill=\"none\" stroke=\"none\"/></svg>";
        let expected = "<svg/>";
        let mut doc = parser::parse(input).unwrap();
        RemoveUselessStrokeAndFill.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_visible() {
        let input = "<svg><rect fill=\"none\" stroke=\"red\"/></svg>";
        // fill none kept because default is black
        let mut doc = parser::parse(input).unwrap();
        RemoveUselessStrokeAndFill.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_remove_zero_width_stroke() {
        let input = "<svg><rect stroke=\"red\" stroke-width=\"0\"/></svg>";
        let expected = "<svg><rect/></svg>"; // stroke removed, default fill (black) remains implied?
                                             // Wait, input has no fill, so default is black.
                                             // Result has no attributes, so default black. Correct.

        // If input was fill="none" stroke="red" width="0"
        // stroke removed -> invisible -> element removed?

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessStrokeAndFill.apply(&mut doc);
        // stroke and stroke-width removed.
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_remove_useless_stroke_attributes_when_stroke_is_none() {
        let input = "<svg><path d=\"M0 0\" stroke=\"none\" stroke-width=\"4\" stroke-linecap=\"round\" stroke-opacity=\".5\" fill=\"red\"/></svg>";
        let expected = "<svg><path d=\"M0 0\" fill=\"red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessStrokeAndFill.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_remove_useless_fill_attributes_when_fill_is_none() {
        let input =
            "<svg><path d=\"M0 0\" fill=\"none\" fill-rule=\"evenodd\" fill-opacity=\".5\" stroke=\"red\"/></svg>";
        let expected = "<svg><path d=\"M0 0\" fill=\"none\" stroke=\"red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUselessStrokeAndFill.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }
}
