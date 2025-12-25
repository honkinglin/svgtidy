use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveEmptyText;

impl Plugin for RemoveEmptyText {
    fn apply(&self, doc: &mut Document) {
        // Default xml:space is "default" (which allows removing whitespace), unless "preserve"
        remove_empty_text_from_nodes(&mut doc.root, false);
    }
}

fn remove_empty_text_from_nodes(nodes: &mut Vec<Node>, preserve_nums: bool) {
    nodes.retain(|node| {
        if let Node::Text(text) = node {
            if !preserve_nums && text.trim().is_empty() {
                return false;
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            // Check xml:space attribute
            let mut preserve = preserve_nums;
            if let Some(val) = elem.attributes.get("xml:space") {
                if val == "preserve" {
                    preserve = true;
                } else if val == "default" {
                    preserve = false;
                }
            }

            remove_empty_text_from_nodes(&mut elem.children, preserve);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_empty_text() {
        let input = "<svg>   <rect/>   </svg>";
        let expected = "<svg><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyText.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_preserve_space() {
        let input = "<svg xml:space=\"preserve\">   <rect/>   </svg>";
        // Should keep the whitespace
        let expected = "<svg xml:space=\"preserve\">   <rect/>   </svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyText.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_nested_preserve() {
        let input = "<svg>   <g xml:space=\"preserve\">  </g>   </svg>";
        // Outer whitespace gone, inner kept
        let expected = "<svg><g xml:space=\"preserve\">  </g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveEmptyText.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }
}
