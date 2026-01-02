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
            if elem.name == "script" {
                return false;
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            remove_script_recursive(&mut elem.children);
        }
    }
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
}
