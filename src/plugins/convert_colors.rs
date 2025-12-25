use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct ConvertColors;

impl Plugin for ConvertColors {
    fn apply(&self, doc: &mut Document) {
        convert_colors_in_nodes(&mut doc.root);
    }
}

fn convert_colors_in_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            convert_element_colors(elem);
            convert_colors_in_nodes(&mut elem.children);
        }
    }
}

fn convert_element_colors(elem: &mut Element) {
    // Attributes that contain colors
    let color_attrs = [
        "fill",
        "stroke",
        "stop-color",
        "flood-color",
        "lighting-color",
    ];

    for attr in color_attrs {
        if let Some(val) = elem.attributes.get_mut(attr) {
            let new_val = convert_color(val);
            *val = new_val;
        }
    }

    // Handle style attribute? (Complexity: parsing CSS. Skipping for now as SVGO usually handles this in a separate pass or complex parser)
}

fn convert_color(val: &str) -> String {
    let lower = val.to_lowercase();

    // 1. RGB conversion: rgb(r, g, b) -> #rrggbb
    static RGB_RE: OnceLock<Regex> = OnceLock::new();
    let rgb_re =
        RGB_RE.get_or_init(|| Regex::new(r"rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap());

    if let Some(caps) = rgb_re.captures(&lower) {
        if let (Ok(r), Ok(g), Ok(b)) = (
            caps[1].parse::<u8>(),
            caps[2].parse::<u8>(),
            caps[3].parse::<u8>(),
        ) {
            let hex = to_hex(r, g, b);
            // Check if named color is shorter? (Not strictly required for strict correctness, but good for optimization)
            return hex;
        }
    }

    // 2. Named color to Hex
    // Basic list of common colors that are shorter in Hex (or commonly used)
    // Detailed list from SVGO commonly converts long names to hex.
    // e.g. "black" -> "#000", "white" -> "#fff"
    // "red" -> "#f00" (same length, but hex preferred often for consistency)
    // "rebeccapurple" -> "#663399"

    static COLORS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let colors = COLORS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("black", "#000");
        m.insert("white", "#fff");
        m.insert("red", "#f00");
        m.insert("green", "#008000"); // #008000 is longer than green? No. 7 chars vs 5. Wait. #008000 is 7. green is 5.
                                      // Actually SVGO checks length.
        m.insert("blue", "#00f");
        m.insert("yellow", "#ff0");
        m.insert("cyan", "#0ff");
        m.insert("magenta", "#f0f");
        m.insert("gray", "#808080"); // 7 vs 4.
        m.insert("grey", "#808080");
        m.insert("rebeccapurple", "#663399");
        // ... add more if needed
        m
    });

    if let Some(hex) = colors.get(lower.as_str()) {
        if hex.len() < lower.len() {
            return hex.to_string();
        }
    }

    // 3. #RRGGBB to #RGB
    // e.g. #aa33cc -> #a3c
    if lower.starts_with('#') && lower.len() == 7 {
        let chars: Vec<char> = lower.chars().collect();
        if chars[1] == chars[2] && chars[3] == chars[4] && chars[5] == chars[6] {
            return format!("#{}{}{}", chars[1], chars[3], chars[5]);
        }
    }

    val.to_string()
}

fn to_hex(r: u8, g: u8, b: u8) -> String {
    let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
    // Try minimize
    let chars: Vec<char> = hex.chars().collect();
    if chars[1] == chars[2] && chars[3] == chars[4] && chars[5] == chars[6] {
        return format!("#{}{}{}", chars[1], chars[3], chars[5]);
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_convert_rgb() {
        let input = "<svg fill=\"rgb(255, 0, 0)\"></svg>";
        let expected = "<svg fill=\"#f00\"/>";
        let mut doc = parser::parse(input).unwrap();
        ConvertColors.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_convert_named() {
        let input = "<svg stroke=\"black\" fill=\"rebeccapurple\"></svg>";
        let expected = "<svg stroke=\"#000\" fill=\"#663399\"/>";
        let mut doc = parser::parse(input).unwrap();
        ConvertColors.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_minimize_hex() {
        let input = "<svg fill=\"#ff00ff\"></svg>";
        let expected = "<svg fill=\"#f0f\"/>";
        let mut doc = parser::parse(input).unwrap();
        ConvertColors.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_ignore_valid_short() {
        let input = "<svg fill=\"red\"></svg>";
        // red is #f00 (4 chars). red is 3 chars. Keep red.
        // My map had red -> #f00. wait.
        // Logic: if hex.len() < lower.len(). #f00 is 4. red is 3. So keep red.
        let expected = "<svg fill=\"red\"/>";
        let mut doc = parser::parse(input).unwrap();
        ConvertColors.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);

        // Wait, "red" is shorter than "#f00"? No. "red" is 3. "#f00" is 4.
        // Correct.

        // "black" (5) -> "#000" (4). Convert.
    }
}
