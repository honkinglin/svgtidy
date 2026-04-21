use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveUnknownsAndDefaults;

impl Default for RemoveUnknownsAndDefaults {
    fn default() -> Self {
        Self
    }
}

impl Plugin for RemoveUnknownsAndDefaults {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            let elem_name = elem.name.clone();
            elem.attributes
                .retain(|key, value| !should_remove_default(&elem_name, key, value));
            process_nodes(&mut elem.children);
        }
    }
}

fn should_remove_default(elem_name: &str, key: &str, value: &str) -> bool {
    match key {
        // These presentation defaults are inherited and safe to drop when equal.
        "stroke-width" => is_one(value),
        "stroke-opacity" | "fill-opacity" | "stop-opacity" => value == "1",
        "letter-spacing" | "word-spacing" => value == "normal",
        "dx" | "dy" => is_zero(value) && elem_name == "feOffset",

        // Shape-position defaults are only removed for element types where omission
        // is known to be equivalent. Keep the rules conservative.
        "x" | "y" => is_zero(value) && matches!(elem_name, "rect" | "image" | "use"),
        "cx" | "cy" => is_zero(value) && matches!(elem_name, "circle" | "ellipse"),
        "r" => is_zero(value) && elem_name == "circle",
        "rx" | "ry" => is_zero(value) && matches!(elem_name, "rect" | "ellipse"),

        _ => false,
    }
}

fn is_zero(value: &str) -> bool {
    matches!(value, "0" | "0px" | "0pt" | "0em")
}

fn is_one(value: &str) -> bool {
    matches!(value, "1" | "1px")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_defaults() {
        let input = "<svg><rect x=\"0\" y=\"0\" width=\"100\" stroke-width=\"1\"/></svg>";
        let expected = "<svg><rect width=\"100\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUnknownsAndDefaults.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_preserve_custom_scale_attr() {
        let input = "<svg><g scale=\"1\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUnknownsAndDefaults.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_preserve_custom_rotate_attr() {
        let input = "<svg><g rotate=\"0\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUnknownsAndDefaults.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_preserve_zero_x_on_unlisted_element() {
        let input = "<svg><filter><feOffset x=\"0\"/></filter></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUnknownsAndDefaults.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_remove_zero_dx_on_fe_offset() {
        let input = "<svg><filter><feOffset dx=\"0\" dy=\"2\"/></filter></svg>";
        let expected = "<svg><filter><feOffset dy=\"2\"/></filter></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveUnknownsAndDefaults.apply(&mut doc);

        assert_eq!(printer::print(&doc), expected);
    }
}
