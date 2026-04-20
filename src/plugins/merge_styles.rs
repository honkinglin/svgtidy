use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};

pub struct MergeStyles;

impl Plugin for MergeStyles {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes.iter_mut() {
        if let Node::Element(elem) = node {
            process_nodes(&mut elem.children);
        }
    }

    merge_style_siblings(nodes);
}

fn merge_style_siblings(nodes: &mut Vec<Node>) {
    let mut merged = Vec::with_capacity(nodes.len());

    for node in nodes.drain(..) {
        match node {
            Node::Element(elem) if elem.name == "style" => {
                if let Some(Node::Element(previous)) = merged.last_mut() {
                    if previous.name == "style"
                        && previous.attributes == elem.attributes
                        && can_merge_style_element(previous)
                        && can_merge_style_element(&elem)
                    {
                        append_style_contents(previous, &elem);
                        continue;
                    }
                }

                merged.push(Node::Element(elem));
            }
            other => merged.push(other),
        }
    }

    *nodes = merged;
}

fn can_merge_style_element(elem: &Element) -> bool {
    elem.children
        .iter()
        .all(|child| matches!(child, Node::Text(_) | Node::Cdata(_)))
}

fn append_style_contents(target: &mut Element, source: &Element) {
    let target_text = collect_style_text(&target.children);
    let source_text = collect_style_text(&source.children);

    let joined = match (target_text.is_empty(), source_text.is_empty()) {
        (true, true) => String::new(),
        (true, false) => source_text,
        (false, true) => target_text,
        (false, false) => format!("{target_text}\n{source_text}"),
    };

    target.children.clear();
    if !joined.is_empty() {
        target.children.push(Node::Text(joined));
    }
}

fn collect_style_text(children: &[Node]) -> String {
    let mut out = String::new();

    for child in children {
        match child {
            Node::Text(text) | Node::Cdata(text) => out.push_str(text),
            _ => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_merge_adjacent_styles() {
        let input = "<svg><style>.a{fill:red}</style><style>.b{fill:blue}</style><rect/></svg>";
        let expected = "<svg><style>.a{fill:red}\n.b{fill:blue}</style><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        MergeStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_do_not_merge_styles_with_different_attributes() {
        let input =
            "<svg><style media=\"screen\">.a{fill:red}</style><style>.b{fill:blue}</style></svg>";

        let mut doc = parser::parse(input).unwrap();
        MergeStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_merge_nested_styles_per_parent() {
        let input = "<svg><defs><style>.a{fill:red}</style><style>.b{fill:blue}</style></defs><style>.c{fill:green}</style></svg>";
        let expected =
            "<svg><defs><style>.a{fill:red}\n.b{fill:blue}</style></defs><style>.c{fill:green}</style></svg>";

        let mut doc = parser::parse(input).unwrap();
        MergeStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_do_not_merge_non_adjacent_styles() {
        let input = "<svg><style>.a{fill:red}</style><rect/><style>.b{fill:blue}</style></svg>";

        let mut doc = parser::parse(input).unwrap();
        MergeStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
