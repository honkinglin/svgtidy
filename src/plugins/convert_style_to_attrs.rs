use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct ConvertStyleToAttrs;

impl Plugin for ConvertStyleToAttrs {
    fn apply(&self, doc: &mut Document) {
        process_style(&mut doc.root);
    }
}

fn process_style(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if let Some(style_val) = elem.attributes.get("style").cloned() {
                if let Some(declarations) = parse_style(&style_val) {
                    let mut remaining = Vec::new();

                    for (key, value) in declarations {
                        if should_convert_property(&key, &value) {
                            elem.attributes.insert(key, value);
                        } else {
                            remaining.push((key, value));
                        }
                    }

                    if remaining.is_empty() {
                        elem.attributes.shift_remove("style");
                    } else {
                        elem.attributes
                            .insert("style".to_string(), format_style(&remaining));
                    }
                }
            }

            process_style(&mut elem.children);
        }
    }
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

fn should_convert_property(key: &str, value: &str) -> bool {
    if key.starts_with("--") || value.contains("!important") {
        return false;
    }

    matches!(
        key,
        "clip-path"
            | "clip-rule"
            | "color"
            | "display"
            | "fill"
            | "fill-opacity"
            | "fill-rule"
            | "filter"
            | "mask"
            | "opacity"
            | "stop-color"
            | "stop-opacity"
            | "stroke"
            | "stroke-dasharray"
            | "stroke-dashoffset"
            | "stroke-linecap"
            | "stroke-linejoin"
            | "stroke-miterlimit"
            | "stroke-opacity"
            | "stroke-width"
            | "visibility"
    )
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
    fn test_convert_style() {
        let input = "<svg><rect style=\"fill: red; stroke: blue\" width=\"10\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertStyleToAttrs.apply(&mut doc);
        let out = printer::print(&doc);

        assert_eq!(
            out,
            "<svg><rect width=\"10\" fill=\"red\" stroke=\"blue\"/></svg>"
        );
    }

    #[test]
    fn test_preserve_important_style() {
        let input = "<svg><rect style=\"fill: red !important; stroke: blue\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertStyleToAttrs.apply(&mut doc);

        assert_eq!(
            printer::print(&doc),
            "<svg><rect style=\"fill: red !important\" stroke=\"blue\"/></svg>"
        );
    }

    #[test]
    fn test_preserve_custom_property() {
        let input = "<svg><rect style=\"--brand: #fff; fill: red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertStyleToAttrs.apply(&mut doc);

        assert_eq!(
            printer::print(&doc),
            "<svg><rect style=\"--brand: #fff\" fill=\"red\"/></svg>"
        );
    }

    #[test]
    fn test_preserve_non_presentation_property() {
        let input = "<svg><rect style=\"transform: rotate(45deg); fill: red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertStyleToAttrs.apply(&mut doc);

        assert_eq!(
            printer::print(&doc),
            "<svg><rect style=\"transform: rotate(45deg)\" fill=\"red\"/></svg>"
        );
    }

    #[test]
    fn test_keep_style_when_parsing_is_unsafe() {
        let input = "<svg><rect style=\"fill: red; broken-decl\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertStyleToAttrs.apply(&mut doc);

        assert_eq!(printer::print(&doc), input);
    }
}
