use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct CleanupEnableBackground;

impl Plugin for CleanupEnableBackground {
    fn apply(&self, doc: &mut Document) {
        if document_uses_filter(&doc.root) {
            return;
        }

        process_nodes(&mut doc.root);
    }
}

fn document_uses_filter(nodes: &[Node]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Element(elem) => elem.name == "filter" || document_uses_filter(&elem.children),
        _ => false,
    })
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            cleanup_enable_background_attr(elem);
            cleanup_enable_background_style(elem);
            process_nodes(&mut elem.children);
        }
    }
}

fn cleanup_enable_background_attr(elem: &mut crate::tree::Element) {
    let Some(value) = elem.attributes.get("enable-background").cloned() else {
        return;
    };

    match cleanup_enable_background_value(elem, &value) {
        Cleanup::Remove => {
            elem.attributes.shift_remove("enable-background");
        }
        Cleanup::Replace(new_value) => {
            elem.attributes
                .insert("enable-background".to_string(), new_value);
        }
        Cleanup::Keep => {}
    }
}

fn cleanup_enable_background_style(elem: &mut crate::tree::Element) {
    let Some(style) = elem.attributes.get("style").cloned() else {
        return;
    };

    let Some(declarations) = parse_style(&style) else {
        return;
    };

    let mut changed = false;
    let mut rewritten = Vec::with_capacity(declarations.len());

    for (name, value) in declarations {
        if name == "enable-background" {
            match cleanup_enable_background_value(elem, &value) {
                Cleanup::Remove => {
                    changed = true;
                }
                Cleanup::Replace(new_value) => {
                    changed = true;
                    rewritten.push((name, new_value));
                }
                Cleanup::Keep => rewritten.push((name, value)),
            }
        } else {
            rewritten.push((name, value));
        }
    }

    if !changed {
        return;
    }

    if rewritten.is_empty() {
        elem.attributes.shift_remove("style");
    } else {
        elem.attributes
            .insert("style".to_string(), format_style(&rewritten));
    }
}

enum Cleanup {
    Remove,
    Replace(String),
    Keep,
}

fn cleanup_enable_background_value(elem: &crate::tree::Element, value: &str) -> Cleanup {
    let Some((x, y, width, height)) = parse_enable_background(value) else {
        return Cleanup::Keep;
    };

    if x != "0" || y != "0" {
        return Cleanup::Keep;
    }

    let Some(elem_width) = normalized_attr(elem, "width") else {
        return Cleanup::Keep;
    };
    let Some(elem_height) = normalized_attr(elem, "height") else {
        return Cleanup::Keep;
    };

    if width != elem_width || height != elem_height {
        return Cleanup::Keep;
    }

    match elem.name.as_str() {
        "svg" => Cleanup::Remove,
        "mask" | "pattern" => Cleanup::Replace("new".to_string()),
        _ => Cleanup::Keep,
    }
}

fn parse_enable_background(value: &str) -> Option<(String, String, String, String)> {
    let normalized = value.replace(',', " ");
    let parts: Vec<&str> = normalized.split_whitespace().collect();

    match parts.as_slice() {
        ["new", x, y, width, height] => Some((
            normalize_length_token(x)?.to_string(),
            normalize_length_token(y)?.to_string(),
            normalize_length_token(width)?.to_string(),
            normalize_length_token(height)?.to_string(),
        )),
        _ => None,
    }
}

fn normalized_attr(elem: &crate::tree::Element, name: &str) -> Option<String> {
    elem.attributes
        .get(name)
        .and_then(|value| normalize_length_token(value))
        .map(str::to_string)
}

fn normalize_length_token(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(number) = trimmed.strip_suffix("px") {
        let number = number.trim();
        if !number.is_empty() {
            return Some(number);
        }
    }

    Some(trimmed)
}

fn parse_style(s: &str) -> Option<Vec<(String, String)>> {
    let mut declarations = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut quote: Option<char> = None;

    for (idx, ch) in s.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
            }
            ';' if paren_depth == 0 => {
                parse_declaration(&s[start..idx], &mut declarations)?;
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }

    if quote.is_some() || paren_depth != 0 {
        return None;
    }

    parse_declaration(&s[start..], &mut declarations)?;
    Some(declarations)
}

fn parse_declaration(raw: &str, declarations: &mut Vec<(String, String)>) -> Option<()> {
    let declaration = raw.trim();
    if declaration.is_empty() {
        return Some(());
    }

    let (key, value) = declaration.split_once(':')?;
    let key = key.trim();
    let value = value.trim();

    if key.is_empty() || value.is_empty() {
        return None;
    }

    declarations.push((key.to_string(), value.to_string()));
    Some(())
}

fn format_style(declarations: &[(String, String)]) -> String {
    declarations
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_remove_enable_background_on_svg() {
        let input =
            "<svg width=\"100\" height=\"50\" enable-background=\"new 0 0 100 50\"><rect/></svg>";
        let expected = "<svg width=\"100\" height=\"50\"><rect/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupEnableBackground.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_replace_enable_background_on_mask() {
        let input =
            "<svg><mask width=\"100\" height=\"50\" enable-background=\"new 0 0 100 50\"/></svg>";
        let expected = "<svg><mask width=\"100\" height=\"50\" enable-background=\"new\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupEnableBackground.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_cleanup_enable_background_in_style_attr() {
        let input = "<svg width=\"100\" height=\"50\" style=\"enable-background: new 0 0 100 50; fill: red\"/>";
        let expected = "<svg width=\"100\" height=\"50\" style=\"fill: red\"/>";

        let mut doc = parser::parse(input).unwrap();
        CleanupEnableBackground.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_skip_when_filter_exists() {
        let input = "<svg width=\"100\" height=\"50\" enable-background=\"new 0 0 100 50\"><filter id=\"f\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        CleanupEnableBackground.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
