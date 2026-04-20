use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};

pub struct MergePaths;

impl Plugin for MergePaths {
    fn apply(&self, doc: &mut Document) {
        merge_paths_in_nodes(&mut doc.root);
    }
}

fn merge_paths_in_nodes(nodes: &mut Vec<Node>) {
    // Recurse first
    for node in nodes.iter_mut() {
        if let Node::Element(elem) = node {
            merge_paths_in_nodes(&mut elem.children);
        }
    }

    // Pass to merge adjacent
    let mut i = 0;
    while i < nodes.len() {
        // Look for sequence of Paths
        // We need mutable access.
        // If nodes[i] is path, and nodes[i+1] is path, check compatibility.

        let can_merge = if i + 1 < nodes.len() {
            match (&nodes[i], &nodes[i + 1]) {
                (Node::Element(e1), Node::Element(e2)) => can_merge_paths(e1, e2),
                _ => false,
            }
        } else {
            false
        };

        if can_merge {
            // Merge i+1 into i
            let d2 = if let Node::Element(e2) = nodes.remove(i + 1) {
                e2.attributes.get("d").cloned().unwrap_or_default()
            } else {
                unreachable!()
            };

            if let Node::Element(e1) = &mut nodes[i] {
                if let Some(d1) = e1.attributes.get_mut("d") {
                    d1.push(' ');
                    d1.push_str(&d2);
                }
            }
            // Don't increment i, check again with new neighbor
        } else {
            i += 1;
        }
    }
}

fn can_merge_paths(e1: &Element, e2: &Element) -> bool {
    if e1.name != "path" || e2.name != "path" {
        return false;
    }

    // Be deliberately conservative: merging paths can affect CSS/marker/bbox-based
    // semantics even when attribute maps look identical. Restrict merges to the
    // simplest case where both elements only carry path data.
    if e1.attributes.len() != 1 || e2.attributes.len() != 1 {
        return false;
    }

    let d1 = e1.attributes.get("d");
    let d2 = e2.attributes.get("d");

    match (d1, d2) {
        (Some(d1), Some(d2)) => !d1.is_empty() && !d2.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_merge_paths() {
        let input = "<svg><path d=\"M0 0L10 10\"/><path d=\"M20 20L30 30\"/></svg>";
        // Should merge
        let expected_d = "M0 0L10 10 M20 20L30 30";

        let mut doc = parser::parse(input).unwrap();
        MergePaths.apply(&mut doc);
        let out = printer::print(&doc);

        assert!(out.contains(expected_d));
        // Should have only 1 path
        assert_eq!(out.matches("<path").count(), 1);
    }

    #[test]
    fn test_no_merge_diff_attrs() {
        let input = "<svg><path d=\"M0 0\" fill=\"red\"/><path d=\"M10 10\" fill=\"blue\"/></svg>";
        let mut doc = parser::parse(input).unwrap();
        MergePaths.apply(&mut doc);
        let out = printer::print(&doc);
        assert_eq!(out.matches("<path").count(), 2);
    }

    #[test]
    fn test_no_merge_when_paths_have_same_presentational_attrs() {
        let input = "<svg><path d=\"M0 0\" fill=\"red\"/><path d=\"M10 10\" fill=\"red\"/></svg>";
        let mut doc = parser::parse(input).unwrap();
        MergePaths.apply(&mut doc);
        let out = printer::print(&doc);
        assert_eq!(out.matches("<path").count(), 2);
    }

    #[test]
    fn test_no_merge_when_marker_is_present() {
        let input =
            "<svg><path d=\"M0 0\" marker-start=\"url(#m)\"/><path d=\"M10 10\" marker-start=\"url(#m)\"/></svg>";
        let mut doc = parser::parse(input).unwrap();
        MergePaths.apply(&mut doc);
        let out = printer::print(&doc);
        assert_eq!(out.matches("<path").count(), 2);
    }
}
