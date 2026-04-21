use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};
use std::collections::{HashMap, HashSet};

pub struct InlineStyles;

impl Plugin for InlineStyles {
    fn apply(&self, doc: &mut Document) {
        let style_plans = collect_style_plans(&doc.root);
        if style_plans.is_empty() {
            return;
        }

        let selectors: HashSet<SimpleSelector> = style_plans
            .iter()
            .flat_map(|plan| {
                plan.rules
                    .iter()
                    .flat_map(|rule| rule.selectors.iter().cloned())
            })
            .collect();

        let matches = collect_selector_matches(&doc.root, &selectors);

        let mut inline_actions: HashMap<Vec<usize>, Vec<Vec<(String, String)>>> = HashMap::new();
        let mut style_updates: HashMap<Vec<usize>, Option<String>> = HashMap::new();
        let mut inlined_classes: HashMap<Vec<usize>, Vec<String>> = HashMap::new();
        let mut remaining_classes = HashSet::new();

        for plan in style_plans {
            let mut remaining_rules = Vec::new();

            for rule in plan.rules {
                let mut remaining_selectors = Vec::new();

                for selector in &rule.selectors {
                    let Some(paths) = matches.get(selector) else {
                        if let SimpleSelector::Class(name) = selector {
                            remaining_classes.insert(name.clone());
                        }
                        remaining_selectors.push(selector.clone());
                        continue;
                    };

                    if paths.len() == 1 {
                        inline_actions
                            .entry(paths[0].clone())
                            .or_default()
                            .push(rule.declarations.clone());
                        if let SimpleSelector::Class(name) = selector {
                            inlined_classes
                                .entry(paths[0].clone())
                                .or_default()
                                .push(name.clone());
                        }
                    } else {
                        if let SimpleSelector::Class(name) = selector {
                            remaining_classes.insert(name.clone());
                        }
                        remaining_selectors.push(selector.clone());
                    }
                }

                if !remaining_selectors.is_empty() {
                    remaining_rules.push(format_rule(&remaining_selectors, &rule.declarations));
                }
            }

            if remaining_rules.is_empty() {
                style_updates.insert(plan.path, None);
            } else {
                style_updates.insert(plan.path, Some(remaining_rules.join("")));
            }
        }

        apply_inline_actions(&mut doc.root, &inline_actions, &mut Vec::new());
        remove_inlined_classes(
            &mut doc.root,
            &inlined_classes,
            &remaining_classes,
            &mut Vec::new(),
        );
        rewrite_style_elements(&mut doc.root, &style_updates, &mut Vec::new());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SimpleSelector {
    Tag(String),
    Class(String),
    Id(String),
}

struct StylePlan {
    path: Vec<usize>,
    rules: Vec<StyleRule>,
}

struct StyleRule {
    selectors: Vec<SimpleSelector>,
    declarations: Vec<(String, String)>,
}

fn collect_style_plans(nodes: &[Node]) -> Vec<StylePlan> {
    let mut plans = Vec::new();
    collect_style_plans_inner(nodes, &mut Vec::new(), &mut plans);
    plans
}

fn collect_style_plans_inner(nodes: &[Node], path: &mut Vec<usize>, plans: &mut Vec<StylePlan>) {
    for (index, node) in nodes.iter().enumerate() {
        path.push(index);

        if let Node::Element(elem) = node {
            if elem.name == "style" {
                if let Some(plan) = build_style_plan(elem, path.clone()) {
                    plans.push(plan);
                }
            } else {
                collect_style_plans_inner(&elem.children, path, plans);
            }
        }

        path.pop();
    }
}

fn build_style_plan(elem: &Element, path: Vec<usize>) -> Option<StylePlan> {
    if !style_attrs_supported(elem) {
        return None;
    }

    let css = collect_style_text(&elem.children)?;
    let rules = parse_stylesheet(&css)?;
    if rules.is_empty() {
        return None;
    }

    Some(StylePlan { path, rules })
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

fn parse_stylesheet(css: &str) -> Option<Vec<StyleRule>> {
    let mut rules = Vec::new();
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut cursor = 0usize;

    while cursor < len {
        while cursor < len && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= len {
            break;
        }

        let selector_start = cursor;
        while cursor < len && bytes[cursor] != b'{' {
            cursor += 1;
        }
        if cursor >= len {
            return None;
        }

        let selector_raw = css[selector_start..cursor].trim();
        cursor += 1;
        let body_start = cursor;
        let mut depth = 1usize;
        let mut quote: Option<u8> = None;

        while cursor < len && depth > 0 {
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
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }

            cursor += 1;
        }

        if depth != 0 {
            return None;
        }

        let body_end = cursor - 1;
        let selectors = parse_selector_list(selector_raw)?;
        let declarations = parse_declarations(&css[body_start..body_end])?;
        if declarations.is_empty() {
            return None;
        }

        rules.push(StyleRule {
            selectors,
            declarations,
        });
    }

    Some(rules)
}

fn parse_selector_list(selector_text: &str) -> Option<Vec<SimpleSelector>> {
    let mut selectors = Vec::new();

    for raw in selector_text.split(',') {
        selectors.push(parse_simple_selector(raw)?);
    }

    (!selectors.is_empty()).then_some(selectors)
}

fn parse_simple_selector(selector: &str) -> Option<SimpleSelector> {
    let selector = selector.trim();
    if selector.is_empty()
        || selector.contains(',')
        || selector.contains(':')
        || selector.contains('[')
        || selector.contains(']')
        || selector.contains('>')
        || selector.contains('+')
        || selector.contains('~')
        || selector.chars().any(char::is_whitespace)
    {
        return None;
    }

    if let Some(name) = selector.strip_prefix('.') {
        return is_ident(name).then(|| SimpleSelector::Class(name.to_string()));
    }

    if let Some(name) = selector.strip_prefix('#') {
        return is_ident(name).then(|| SimpleSelector::Id(name.to_string()));
    }

    is_ident(selector).then(|| SimpleSelector::Tag(selector.to_string()))
}

fn is_ident(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
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

fn collect_selector_matches(
    nodes: &[Node],
    selectors: &HashSet<SimpleSelector>,
) -> HashMap<SimpleSelector, Vec<Vec<usize>>> {
    let mut matches = HashMap::new();
    collect_selector_matches_inner(nodes, selectors, &mut Vec::new(), &mut matches);
    matches
}

fn collect_selector_matches_inner(
    nodes: &[Node],
    selectors: &HashSet<SimpleSelector>,
    path: &mut Vec<usize>,
    matches: &mut HashMap<SimpleSelector, Vec<Vec<usize>>>,
) {
    for (index, node) in nodes.iter().enumerate() {
        path.push(index);

        if let Node::Element(elem) = node {
            for selector in selectors {
                if element_matches_selector(elem, selector) {
                    matches
                        .entry(selector.clone())
                        .or_default()
                        .push(path.clone());
                }
            }

            collect_selector_matches_inner(&elem.children, selectors, path, matches);
        }

        path.pop();
    }
}

fn element_matches_selector(elem: &Element, selector: &SimpleSelector) -> bool {
    match selector {
        SimpleSelector::Tag(name) => elem.name == *name,
        SimpleSelector::Class(name) => elem
            .attributes
            .get("class")
            .is_some_and(|classes| classes.split_whitespace().any(|class| class == name)),
        SimpleSelector::Id(name) => elem.attributes.get("id").is_some_and(|id| id == name),
    }
}

fn apply_inline_actions(
    nodes: &mut Vec<Node>,
    actions: &HashMap<Vec<usize>, Vec<Vec<(String, String)>>>,
    path: &mut Vec<usize>,
) {
    for (index, node) in nodes.iter_mut().enumerate() {
        path.push(index);

        if let Node::Element(elem) = node {
            if let Some(rule_sets) = actions.get(path) {
                let mut declarations = Vec::new();
                for rule in rule_sets {
                    declarations.extend(rule.clone());
                }

                if let Some(existing) = elem.attributes.get("style").cloned() {
                    if let Some(existing_declarations) = parse_declarations(&existing) {
                        declarations.extend(existing_declarations);
                        elem.attributes
                            .insert("style".to_string(), format_style(&declarations));
                    } else {
                        let mut style = format_style(&declarations);
                        if !style.is_empty() && !existing.is_empty() {
                            style.push(';');
                        }
                        style.push_str(&existing);
                        elem.attributes.insert("style".to_string(), style);
                    }
                } else if !declarations.is_empty() {
                    elem.attributes
                        .insert("style".to_string(), format_style(&declarations));
                }
            }

            apply_inline_actions(&mut elem.children, actions, path);
        }

        path.pop();
    }
}

fn rewrite_style_elements(
    nodes: &mut Vec<Node>,
    updates: &HashMap<Vec<usize>, Option<String>>,
    path: &mut Vec<usize>,
) {
    let mut rewritten = Vec::with_capacity(nodes.len());

    for (index, mut node) in nodes.drain(..).enumerate() {
        path.push(index);

        let keep = match &mut node {
            Node::Element(elem) => {
                if let Some(update) = updates.get(path) {
                    match update {
                        Some(css) => {
                            elem.children.clear();
                            if !css.is_empty() {
                                elem.children.push(Node::Text(css.clone()));
                            }
                            true
                        }
                        None => false,
                    }
                } else {
                    rewrite_style_elements(&mut elem.children, updates, path);
                    true
                }
            }
            _ => true,
        };

        path.pop();

        if keep {
            rewritten.push(node);
        }
    }

    *nodes = rewritten;
}

fn remove_inlined_classes(
    nodes: &mut Vec<Node>,
    inlined_classes: &HashMap<Vec<usize>, Vec<String>>,
    remaining_classes: &HashSet<String>,
    path: &mut Vec<usize>,
) {
    for (index, node) in nodes.iter_mut().enumerate() {
        path.push(index);

        if let Node::Element(elem) = node {
            if let Some(class_names) = inlined_classes.get(path) {
                strip_class_names(elem, class_names, remaining_classes);
            }

            remove_inlined_classes(&mut elem.children, inlined_classes, remaining_classes, path);
        }

        path.pop();
    }
}

fn strip_class_names(
    elem: &mut Element,
    class_names: &[String],
    remaining_classes: &HashSet<String>,
) {
    let Some(class_attr) = elem.attributes.get("class").cloned() else {
        return;
    };

    let removable: HashSet<&str> = class_names
        .iter()
        .map(String::as_str)
        .filter(|name| !remaining_classes.contains(*name))
        .collect();
    if removable.is_empty() {
        return;
    }

    let kept: Vec<&str> = class_attr
        .split_whitespace()
        .filter(|class_name| !removable.contains(*class_name))
        .collect();

    if kept.is_empty() {
        elem.attributes.shift_remove("class");
    } else if kept.len() * 2 - 1 < class_attr.len() {
        elem.attributes
            .insert("class".to_string(), kept.join(" "));
    }
}

fn format_rule(selectors: &[SimpleSelector], declarations: &[(String, String)]) -> String {
    format!(
        "{}{{{}}}",
        selectors
            .iter()
            .map(format_selector)
            .collect::<Vec<_>>()
            .join(","),
        format_style(declarations)
    )
}

fn format_selector(selector: &SimpleSelector) -> String {
    match selector {
        SimpleSelector::Tag(name) => name.clone(),
        SimpleSelector::Class(name) => format!(".{name}"),
        SimpleSelector::Id(name) => format!("#{name}"),
    }
}

fn format_style(declarations: &[(String, String)]) -> String {
    let declarations = prune_overridden_declarations(declarations);
    declarations
        .into_iter()
        .map(|(key, value)| format!("{key}:{value}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn prune_overridden_declarations(declarations: &[(String, String)]) -> Vec<(String, String)> {
    let mut seen = HashSet::new();
    let mut kept = Vec::with_capacity(declarations.len());

    for (key, value) in declarations.iter().rev() {
        if can_prune_declaration(key, value) {
            if seen.insert(key.clone()) {
                kept.push((key.clone(), value.clone()));
            }
        } else {
            kept.push((key.clone(), value.clone()));
        }
    }

    kept.reverse();
    kept
}

fn can_prune_declaration(key: &str, value: &str) -> bool {
    !key.starts_with("--") && !value.contains("!important")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_inline_unique_class_selector() {
        let input = "<svg><style>.a{fill:red}</style><rect class=\"a\"/></svg>";
        let expected = "<svg><rect style=\"fill:red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_existing_inline_style_keeps_precedence() {
        let input = "<svg><style>.a{fill:red;stroke:black}</style><rect class=\"a\" style=\"stroke: blue\"/></svg>";
        let expected = "<svg><rect style=\"fill:red;stroke:blue\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_important_declaration_when_merging_styles() {
        let input =
            "<svg><style>.a{stroke:black!important}</style><rect class=\"a\" style=\"stroke:blue\"/></svg>";
        let expected = "<svg><rect style=\"stroke:black!important;stroke:blue\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_do_not_inline_when_selector_matches_multiple_elements() {
        let input =
            "<svg><style>.a{fill:red}</style><rect class=\"a\"/><circle class=\"a\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_do_not_inline_unsupported_selector() {
        let input = "<svg><style>g .a{fill:red}</style><g><rect class=\"a\"/></g></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_remove_only_inlined_rule() {
        let input = "<svg><style>.a{fill:red}.b{stroke:blue}</style><rect class=\"a\"/><rect class=\"b\"/><circle class=\"b\"/></svg>";
        let expected =
            "<svg><style>.b{stroke:blue}</style><rect style=\"fill:red\"/><rect class=\"b\"/><circle class=\"b\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_remove_inlined_class_from_multi_class_attr() {
        let input = "<svg><style>.a{fill:red}</style><rect class=\"a keep\"/></svg>";
        let expected = "<svg><rect class=\"keep\" style=\"fill:red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_class_when_same_selector_remains() {
        let input = "<svg><style>.a{fill:red}.a{stroke:blue}</style><rect class=\"a\"/><circle class=\"a\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_inline_grouped_simple_selectors() {
        let input = "<svg><style>.a,.b{fill:red}</style><rect class=\"a\"/><circle class=\"b\"/></svg>";
        let expected = "<svg><rect style=\"fill:red\"/><circle style=\"fill:red\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_non_inlined_selector_from_group() {
        let input = "<svg><style>.a,.b{fill:red}</style><rect class=\"a\"/><circle class=\"b\"/><path class=\"b\"/></svg>";
        let expected =
            "<svg><style>.b{fill:red}</style><rect style=\"fill:red\"/><circle class=\"b\"/><path class=\"b\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        InlineStyles.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }
}
