use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};

pub struct MinifyStyles;

impl Plugin for MinifyStyles {
    fn apply(&self, doc: &mut Document) {
        process_nodes(&mut doc.root);
    }
}

fn process_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            minify_style_attribute(elem);
            minify_style_element(elem);
            process_nodes(&mut elem.children);
        }
    }
}

fn minify_style_attribute(elem: &mut Element) {
    let Some(style) = elem.attributes.get("style").cloned() else {
        return;
    };

    let Some(declarations) = parse_declarations(&strip_css_comments(&style)) else {
        return;
    };

    elem.attributes
        .insert("style".to_string(), format_declarations(&declarations));
}

fn minify_style_element(elem: &mut Element) {
    if elem.name != "style" || !style_attrs_supported(elem) {
        return;
    }

    let Some(text) = collect_style_text(&elem.children) else {
        return;
    };

    let stripped = strip_css_comments(&text);
    let Some(rules) = parse_stylesheet(&stripped) else {
        return;
    };

    let minified = rules
        .into_iter()
        .map(|(selector, declarations)| {
            format!(
                "{}{{{}}}",
                selector.trim(),
                format_declarations(&declarations)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    elem.children.clear();
    if !minified.is_empty() {
        elem.children.push(Node::Text(minified));
    }
}

fn style_attrs_supported(elem: &Element) -> bool {
    match elem.attributes.len() {
        0 => true,
        1 => elem
            .attributes
            .get("type")
            .is_some_and(|value| value == "text/css"),
        _ => false,
    }
}

fn collect_style_text(children: &[Node]) -> Option<String> {
    let mut out = String::new();

    for child in children {
        match child {
            Node::Text(text) | Node::Cdata(text) => out.push_str(text),
            _ => return None,
        }
    }

    Some(out)
}

fn strip_css_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let chars: Vec<char> = css.chars().collect();
    let mut idx = 0usize;
    let mut quote: Option<char> = None;

    while idx < chars.len() {
        let ch = chars[idx];

        if let Some(active_quote) = quote {
            out.push(ch);
            if ch == active_quote {
                quote = None;
            }
            idx += 1;
            continue;
        }

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            out.push(ch);
            idx += 1;
            continue;
        }

        if ch == '/' && idx + 1 < chars.len() && chars[idx + 1] == '*' {
            idx += 2;
            while idx + 1 < chars.len() && !(chars[idx] == '*' && chars[idx + 1] == '/') {
                idx += 1;
            }
            idx = (idx + 2).min(chars.len());
            continue;
        }

        out.push(ch);
        idx += 1;
    }

    out
}

fn parse_stylesheet(css: &str) -> Option<Vec<(String, Vec<(String, String)>)>> {
    let bytes = css.as_bytes();
    let mut rules = Vec::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let selector_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'{' {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return None;
        }

        let selector = css[selector_start..cursor].trim();
        if selector.is_empty() || selector.starts_with('@') {
            return None;
        }

        cursor += 1;
        let body_start = cursor;
        let mut quote: Option<u8> = None;

        while cursor < bytes.len() {
            let byte = bytes[cursor];

            if let Some(active_quote) = quote {
                if byte == active_quote {
                    quote = None;
                }
                cursor += 1;
                continue;
            }

            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'{' => return None,
                b'}' => break,
                _ => cursor += 1,
            }
        }

        if cursor >= bytes.len() {
            return None;
        }

        let declarations = parse_declarations(&css[body_start..cursor])?;
        rules.push((selector.to_string(), declarations));
        cursor += 1;
    }

    Some(rules)
}

fn parse_declarations(body: &str) -> Option<Vec<(String, String)>> {
    let mut declarations = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut quote: Option<char> = None;

    for (idx, ch) in body.char_indices() {
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
                parse_declaration(&body[start..idx], &mut declarations)?;
                start = idx + 1;
            }
            _ => {}
        }
    }

    if quote.is_some() || paren_depth != 0 {
        return None;
    }

    parse_declaration(&body[start..], &mut declarations)?;
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

fn format_declarations(declarations: &[(String, String)]) -> String {
    declarations
        .iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_minify_style_attribute() {
        let input = "<svg><rect style=\" fill : red ; stroke : blue ; \"/></svg>";
        let expected = "<svg><rect style=\"fill:red;stroke:blue\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        MinifyStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_minify_style_element() {
        let input = "<svg><style> .a { fill : red ; stroke : blue ; } </style></svg>";
        let expected = "<svg><style>.a{fill:red;stroke:blue}</style></svg>";

        let mut doc = parser::parse(input).unwrap();
        MinifyStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_remove_css_comments() {
        let input = "<svg><style>/* comment */ .a { fill: red; }</style></svg>";
        let expected = "<svg><style>.a{fill:red}</style></svg>";

        let mut doc = parser::parse(input).unwrap();
        MinifyStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_skip_unsupported_stylesheet() {
        let input = "<svg><style>@media screen {.a { fill: red; }}</style></svg>";

        let mut doc = parser::parse(input).unwrap();
        MinifyStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }
}
