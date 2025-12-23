use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveComments;

impl Plugin for RemoveComments {
    fn apply(&self, doc: &mut Document) {
        remove_comments_from_nodes(&mut doc.root);
    }
}

fn remove_comments_from_nodes(nodes: &mut Vec<Node>) {
    nodes.retain(|node| !matches!(node, Node::Comment(_)));
    for node in nodes {
        if let Node::Element(elem) = node {
            remove_comments_from_nodes(&mut elem.children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_comments() {
        let input = "<svg><!-- Comment 1 --><rect/><g><!-- Comment 2 --></g></svg>";
        let expected = "<svg><rect/><g/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveComments.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }
}
