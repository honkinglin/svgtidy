use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};

pub struct CleanupNumericValues {
    pub float_precision: usize,
    pub remove_px: bool,
    pub leading_zero: bool,
}

impl Default for CleanupNumericValues {
    fn default() -> Self {
        Self {
            float_precision: 3,
            remove_px: true,
            leading_zero: true,
        }
    }
}

impl Plugin for CleanupNumericValues {
    fn apply(&self, doc: &mut Document) {
        cleanup_numeric_in_nodes(&mut doc.root, self);
    }
}

fn cleanup_numeric_in_nodes(nodes: &mut Vec<Node>, opts: &CleanupNumericValues) {
    for node in nodes {
        if let Node::Element(elem) = node {
            cleanup_element_numeric(elem, opts);
            cleanup_numeric_in_nodes(&mut elem.children, opts);
        }
    }
}

fn cleanup_element_numeric(elem: &mut Element, opts: &CleanupNumericValues) {
    // List of attributes to check
    let numeric_attrs = [
        "x",
        "y",
        "width",
        "height",
        "r",
        "cx",
        "cy",
        "rx",
        "ry",
        "opacity",
        "fill-opacity",
        "stroke-opacity",
        "stroke-width",
        "font-size",
        "offset",
    ];

    for attr in numeric_attrs {
        if let Some(val) = elem.attributes.get_mut(attr) {
            let new_val = cleanup_number(val, opts);
            *val = new_val;
        }
    }
}

fn cleanup_number(val: &str, opts: &CleanupNumericValues) -> String {
    // 1. Remove px
    let mut clean_val = val.trim();
    if opts.remove_px && clean_val.ends_with("px") {
        clean_val = &clean_val[..clean_val.len() - 2];
    }

    // 2. Parse number
    if let Ok(num) = clean_val.parse::<f64>() {
        // 3. Round
        // Format with precision
        let p = opts.float_precision;
        let factor = 10u32.pow(p as u32) as f64;
        let rounded = (num * factor).round() / factor;

        let s = rounded.to_string();

        // 4. Remove leading zero: 0.5 -> .5, -0.5 -> -.5
        if opts.leading_zero {
            if s.starts_with("0.") {
                return s[1..].to_string();
            } else if s.starts_with("-0.") {
                return format!("-{}", &s[2..]);
            }
        }
        return s;
    }

    // Fallback if parse fails (e.g. lists, percentages if parser failed)
    val.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_cleanup_px() {
        let input = "<svg width=\"100px\" height=\"50.5px\"></svg>";
        let expected = "<svg width=\"100\" height=\"50.5\"/>";

        let mut doc = parser::parse(input).unwrap();
        let plugin = CleanupNumericValues::default();
        plugin.apply(&mut doc);

        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_round() {
        // default precision 3
        let input = "<svg opacity=\"0.123456\"></svg>";
        let expected = "<svg opacity=\".123\"/>";

        let mut doc = parser::parse(input).unwrap();
        let plugin = CleanupNumericValues::default();
        plugin.apply(&mut doc);

        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_leading_zero() {
        let input = "<svg opacity=\"0.5\"></svg>";
        let expected = "<svg opacity=\".5\"/>"; // default leading_zero=true

        let mut doc = parser::parse(input).unwrap();
        let plugin = CleanupNumericValues::default();
        plugin.apply(&mut doc);

        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_ignore_non_numeric() {
        let input = "<svg width=\"auto\"></svg>";
        // parse fails, keep original
        let expected = "<svg width=\"auto\"/>";

        let mut doc = parser::parse(input).unwrap();
        let plugin = CleanupNumericValues::default();
        plugin.apply(&mut doc);

        assert_eq!(printer::print(&doc), expected);
    }
}
