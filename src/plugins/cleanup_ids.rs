use crate::plugins::collections::find_used_ids;
use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::collections::HashSet;

pub struct CleanupIds;

impl Plugin for CleanupIds {
    fn apply(&self, doc: &mut Document) {
        let mut used_ids = HashSet::new();
        for node in &doc.root {
            find_used_ids(node, &mut used_ids);
        }

        cleanup_ids_in_nodes(&mut doc.root, &used_ids);
    }
}

fn cleanup_ids_in_nodes(nodes: &mut Vec<Node>, used_ids: &HashSet<String>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            // Check ID
            if let Some(id) = elem.attributes.get("id").cloned() {
                // Clone identifier to avoid borrow issues
                if !used_ids.contains(&id) {
                    elem.attributes.shift_remove("id");
                }
            }

            cleanup_ids_in_nodes(&mut elem.children, used_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_cleanup_unused_ids() {
        let input = "<svg><rect id=\"unused\"/><rect id=\"used\"/><use href=\"#used\"/></svg>";
        let expected = "<svg><rect/><rect id=\"used\"/><use href=\"#used\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_cleanup_ids_url_ref() {
        let input = "<svg><linearGradient id=\"grad\"/><rect fill=\"url(#grad)\"/></svg>";
        let expected = "<svg><linearGradient id=\"grad\"/><rect fill=\"url(#grad)\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_cleanup_ids_preserve_aria_idrefs() {
        let input = "<svg><title id=\"title\">Name</title><rect aria-labelledby=\"title\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert!(output.contains("<title id=\"title\">"));
    }

    #[test]
    fn test_cleanup_ids_preserve_begin_refs() {
        let input = "<svg><path id=\"p\"/><animate href=\"#p\" begin=\"p.end\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert!(output.contains("<path id=\"p\"/>"));
    }
}
