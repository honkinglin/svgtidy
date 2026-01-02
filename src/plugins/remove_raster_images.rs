use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveRasterImages;

impl Plugin for RemoveRasterImages {
    fn apply(&self, doc: &mut Document) {
        remove_images_recursive(&mut doc.root);
    }
}

fn remove_images_recursive(nodes: &mut Vec<Node>) {
    nodes.retain(|node| {
        if let Node::Element(elem) = node {
            if elem.name == "image" || elem.name == "img" {
                return false;
            }
        }
        true
    });

    for node in nodes {
        if let Node::Element(elem) = node {
            remove_images_recursive(&mut elem.children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_image() {
        let input = "<svg><image href=\"foo.jpg\"/><rect/></svg>";
        let expected = "<svg><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveRasterImages.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }
}
