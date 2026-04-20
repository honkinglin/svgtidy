use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct RemoveNonInheritableGroupAttrs;

impl Plugin for RemoveNonInheritableGroupAttrs {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "g" {
                elem.attributes
                    .retain(|name, _| !NON_INHERITABLE_GROUP_ATTRS.contains(&name.as_str()));
            }
            process_nodes(&mut elem.children);
        }
    }
}

// Conservative subset of SVGO's removeNonInheritableGroupAttrs:
// these presentation attrs do not inherit and do not apply to <g> itself.
const NON_INHERITABLE_GROUP_ATTRS: &[&str] = &[
    "clip-rule",
    "fill-rule",
    "flood-color",
    "flood-opacity",
    "lighting-color",
    "stop-color",
    "stop-opacity",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_non_inheritable_group_attrs() {
        let input = "<svg><g clip-rule=\"evenodd\" fill-rule=\"evenodd\" stop-color=\"#fff\"><rect/></g></svg>";
        let expected = "<svg><g><rect/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveNonInheritableGroupAttrs.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_inheritable_group_attrs() {
        let input = "<svg><g fill=\"red\" stroke=\"blue\"><rect/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveNonInheritableGroupAttrs.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_keep_group_attrs_that_apply_to_group() {
        let input = "<svg><g opacity=\"0.5\" filter=\"url(#f)\" mask=\"url(#m)\"><rect/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        RemoveNonInheritableGroupAttrs.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
