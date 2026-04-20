use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveScriptElement;

impl Plugin for RemoveScriptElement {
    fn apply(&self, doc: &mut Document) {
        remove_script_recursive(&mut doc.root);
    }
}

fn remove_script_recursive(nodes: &mut Vec<Node>) {
    nodes.retain(|node| {
        if let Node::Element(elem) = node {
            if elem.name == "script" || elem.name == "foreignObject" {
                return false;
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            elem.attributes
                .retain(|key, value| !is_script_attr(key, value));
            remove_script_recursive(&mut elem.children);
        }
    }
}

fn is_script_attr(key: &str, value: &str) -> bool {
    if key.to_ascii_lowercase().starts_with("on") {
        return true;
    }

    let normalized = value.trim_start().to_ascii_lowercase();
    normalized.starts_with("javascript:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_script() {
        let input = "<svg><script>alert(1)</script><rect/></svg>";
        let expected = "<svg><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveScriptElement.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_remove_script_attrs() {
        let input =
            "<svg><rect onclick=\"alert(1)\" href=\"javascript:alert(1)\" width=\"10\"/></svg>";
        let expected = "<svg><rect width=\"10\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveScriptElement.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_remove_foreign_object() {
        let input = "<svg><foreignObject><div>unsafe</div></foreignObject><rect/></svg>";
        let expected = "<svg><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveScriptElement.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }
}
