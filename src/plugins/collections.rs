use crate::tree::Node;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

pub fn find_used_ids(node: &Node, used_ids: &mut HashSet<String>) {
    match node {
        Node::Element(elem) => {
            // Check all attributes for references
            for (key, value) in &elem.attributes {
                extract_ids_from_attr(key, value, used_ids);
            }

            // Recurse
            for child in &elem.children {
                find_used_ids(child, used_ids);
            }
        }
        _ => {}
    }
}

pub fn node_has_used_id(node: &Node, used_ids: &HashSet<String>) -> bool {
    match node {
        Node::Element(elem) => {
            if elem
                .attributes
                .get("id")
                .is_some_and(|id| used_ids.contains(id))
            {
                return true;
            }

            elem.children
                .iter()
                .any(|child| node_has_used_id(child, used_ids))
        }
        _ => false,
    }
}

fn extract_ids_from_value(value: &str, used_ids: &mut HashSet<String>) {
    // 1. url(#id)
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let url_re = URL_RE.get_or_init(|| Regex::new(r"url\s*\(\s*#([^\s\)]+)\s*\)").unwrap());

    for cap in url_re.captures_iter(value) {
        if let Some(id) = cap.get(1) {
            used_ids.insert(id.as_str().to_string());
        }
    }

    // 2. plain #id references such as href, xlink:href, begin/end and ARIA idrefs
    static HASH_REF_RE: OnceLock<Regex> = OnceLock::new();
    let hash_ref_re =
        HASH_REF_RE.get_or_init(|| Regex::new(r"#([A-Za-z_][A-Za-z0-9_.:-]*)").unwrap());

    for cap in hash_ref_re.captures_iter(value) {
        if let Some(id) = cap.get(1) {
            used_ids.insert(id.as_str().to_string());
        }
    }
}

fn extract_ids_from_attr(key: &str, value: &str, used_ids: &mut HashSet<String>) {
    extract_ids_from_value(value, used_ids);

    let is_idref_list_attr = matches!(
        key,
        "aria-labelledby"
            | "aria-describedby"
            | "aria-owns"
            | "aria-controls"
            | "aria-flowto"
            | "aria-activedescendant"
    );

    if !is_idref_list_attr {
        return;
    }

    for token in value
        .split(|c: char| c.is_ascii_whitespace() || c == ',' || c == ';')
        .filter(|token| !token.is_empty())
    {
        used_ids.insert(token.to_string());
    }
}
