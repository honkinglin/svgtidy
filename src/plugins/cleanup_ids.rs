use crate::plugins::collections::find_used_ids;
use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::collections::{HashMap, HashSet};

pub struct CleanupIds;

impl Plugin for CleanupIds {
    fn apply(&self, doc: &mut Document) {
        if has_style_element(&doc.root) {
            return;
        }

        let mut used_ids = HashSet::new();
        for node in &doc.root {
            find_used_ids(node, &mut used_ids);
        }

        let rename_map = build_rename_map(&doc.root, &used_ids);

        cleanup_ids_in_nodes(&mut doc.root, &used_ids, &rename_map);
    }
}

fn cleanup_ids_in_nodes(
    nodes: &mut Vec<Node>,
    used_ids: &HashSet<String>,
    rename_map: &HashMap<String, String>,
) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if let Some(id) = elem.attributes.get("id").cloned() {
                if !used_ids.contains(&id) {
                    elem.attributes.shift_remove("id");
                } else if let Some(new_id) = rename_map.get(&id) {
                    elem.attributes.insert("id".to_string(), new_id.clone());
                }
            }

            for (key, value) in elem.attributes.iter_mut() {
                if key == "id" {
                    continue;
                }
                *value = rewrite_attr_value(key, value, rename_map);
            }

            cleanup_ids_in_nodes(&mut elem.children, used_ids, rename_map);
        }
    }
}

fn build_rename_map(nodes: &[Node], used_ids: &HashSet<String>) -> HashMap<String, String> {
    let mut ordered = Vec::new();
    collect_used_ids_in_order(nodes, used_ids, &mut ordered);

    let mut map = HashMap::new();
    for (index, id) in ordered.into_iter().enumerate() {
        let candidate = short_id(index);
        if candidate.len() < id.len() {
            map.insert(id, candidate);
        }
    }
    map
}

fn collect_used_ids_in_order(nodes: &[Node], used_ids: &HashSet<String>, out: &mut Vec<String>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if let Some(id) = elem.attributes.get("id") {
                if used_ids.contains(id) && !out.contains(id) {
                    out.push(id.clone());
                }
            }

            collect_used_ids_in_order(&elem.children, used_ids, out);
        }
    }
}

fn short_id(mut index: usize) -> String {
    const ALPHABET: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
    let mut out = String::new();

    loop {
        out.insert(0, ALPHABET[index % 26] as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }

    out
}

fn has_style_element(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Element(elem) => elem.name == "style" || has_style_element(&elem.children),
        _ => false,
    })
}

fn rewrite_attr_value(key: &str, value: &str, rename_map: &HashMap<String, String>) -> String {
    if rename_map.is_empty() {
        return value.to_string();
    }

    if matches!(
        key,
        "aria-labelledby"
            | "aria-describedby"
            | "aria-owns"
            | "aria-controls"
            | "aria-flowto"
            | "aria-activedescendant"
    ) {
        return value
            .split_whitespace()
            .map(|token| rename_map.get(token).cloned().unwrap_or_else(|| token.to_string()))
            .collect::<Vec<_>>()
            .join(" ");
    }

    if matches!(key, "begin" | "end") {
        return rewrite_timing_refs(value, rename_map);
    }

    let mut rewritten = value.to_string();
    for (old_id, new_id) in rename_map {
        rewritten = rewritten.replace(&format!("url(#{old_id})"), &format!("url(#{new_id})"));
        rewritten = rewritten.replace(&format!("#{old_id}"), &format!("#{new_id}"));
    }
    rewritten
}

fn rewrite_timing_refs(value: &str, rename_map: &HashMap<String, String>) -> String {
    value
        .split(';')
        .map(|part| {
            let trimmed = part.trim();
            if let Some((id, suffix)) = trimmed.split_once('.') {
                if let Some(new_id) = rename_map.get(id) {
                    return format!("{new_id}.{suffix}");
                }
            }
            trimmed.to_string()
        })
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_cleanup_unused_ids() {
        let input = "<svg><rect id=\"unused\"/><rect id=\"used\"/><use href=\"#used\"/></svg>";
        let expected = "<svg><rect/><rect id=\"a\"/><use href=\"#a\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_cleanup_ids_url_ref() {
        let input = "<svg><linearGradient id=\"grad\"/><rect fill=\"url(#grad)\"/></svg>";
        let expected = "<svg><linearGradient id=\"a\"/><rect fill=\"url(#a)\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_cleanup_ids_preserve_aria_idrefs() {
        let input = "<svg><title id=\"title\">Name</title><rect aria-labelledby=\"title\"/></svg>";
        let expected = "<svg><title id=\"a\">Name</title><rect aria-labelledby=\"a\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_cleanup_ids_preserve_begin_refs() {
        let input = "<svg><path id=\"p\"/><animate href=\"#p\" begin=\"p.end\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);
        let output = printer::print(&doc);

        assert_eq!(output, input);
    }

    #[test]
    fn test_cleanup_ids_skip_renaming_when_style_element_exists() {
        let input = "<svg><style>#hero{fill:red}</style><path id=\"hero\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupIds.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }
}
