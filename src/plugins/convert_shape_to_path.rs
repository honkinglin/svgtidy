use crate::plugins::Plugin;
use crate::tree::{Document, Element, Node};

pub struct ConvertShapeToPath;

impl Plugin for ConvertShapeToPath {
    fn apply(&self, doc: &mut Document) {
        convert_shapes_in_nodes(&mut doc.root);
    }
}

fn convert_shapes_in_nodes(nodes: &mut Vec<Node>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            convert_element(elem);
            convert_shapes_in_nodes(&mut elem.children);
        }
    }
}

fn convert_element(elem: &mut Element) {
    let d = match elem.name.as_str() {
        "rect" => convert_rect(elem),
        "line" => convert_line(elem),
        "polygon" => convert_poly(elem, true),
        "polyline" => convert_poly(elem, false),
        _ => None,
    };

    if let Some(path_data) = d {
        // Change tag to path
        elem.name = "path".to_string();
        // Remove shape attributes
        remove_shape_attrs(elem);
        // Add d attribute
        elem.attributes.insert("d".to_string(), path_data);
    }
}

fn get_num(elem: &Element, attr: &str, def: f64) -> f64 {
    elem.attributes
        .get(attr)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(def)
}

fn convert_rect(elem: &Element) -> Option<String> {
    let w = get_num(elem, "width", 0.0);
    let h = get_num(elem, "height", 0.0);
    let x = get_num(elem, "x", 0.0);
    let y = get_num(elem, "y", 0.0);
    let rx_raw = elem
        .attributes
        .get("rx")
        .and_then(|v| v.parse::<f64>().ok());
    let ry_raw = elem
        .attributes
        .get("ry")
        .and_then(|v| v.parse::<f64>().ok());

    if w == 0.0 || h == 0.0 {
        return None; // Invalid or invisible
    }

    if rx_raw.is_none() && ry_raw.is_none() {
        // M x y H x+w V y+h H x z
        // Or simpler: M x y h w v h h -w z
        // Let's use absolute for simplicity initially
        return Some(format!("M{} {}h{}v{}h-{}z", x, y, w, h, w));
    }

    let (rx, ry) = resolve_rect_radius(rx_raw, ry_raw, w, h)?;

    if rx == 0.0 || ry == 0.0 {
        return Some(format!("M{} {}h{}v{}h-{}z", x, y, w, h, w));
    }

    let horizontal = w - 2.0 * rx;
    let vertical = h - 2.0 * ry;

    let path_data = format!(
        "M{} {}h{}a{} {} 0 0 1 {} {}v{}a{} {} 0 0 1 -{} {}h-{}a{} {} 0 0 1 -{} -{}v-{}a{} {} 0 0 1 {} -{}z",
        x + rx,
        y,
        horizontal,
        rx,
        ry,
        rx,
        ry,
        vertical,
        rx,
        ry,
        rx,
        ry,
        horizontal,
        rx,
        ry,
        rx,
        ry,
        vertical,
        rx,
        ry,
        rx,
        ry
    );

    if serialized_attr_len("d", &path_data)
        >= serialized_present_shape_attrs_len(elem, &["x", "y", "width", "height", "rx", "ry"])
    {
        return None;
    }

    Some(path_data)
}

fn resolve_rect_radius(rx_raw: Option<f64>, ry_raw: Option<f64>, w: f64, h: f64) -> Option<(f64, f64)> {
    let mut rx = match (rx_raw, ry_raw) {
        (Some(rx), _) => rx,
        (None, Some(ry)) => ry,
        (None, None) => 0.0,
    };
    let mut ry = match (ry_raw, rx_raw) {
        (Some(ry), _) => ry,
        (None, Some(rx)) => rx,
        (None, None) => 0.0,
    };

    if rx < 0.0 || ry < 0.0 {
        return None;
    }

    rx = rx.min(w / 2.0);
    ry = ry.min(h / 2.0);

    Some((rx, ry))
}

fn serialized_present_shape_attrs_len(elem: &Element, attrs: &[&str]) -> usize {
    attrs.iter()
        .filter_map(|attr| elem.attributes.get(*attr).map(|value| serialized_attr_len(attr, value)))
        .sum()
}

fn serialized_attr_len(name: &str, value: &str) -> usize {
    name.len() + value.len() + 4
}

fn convert_line(elem: &Element) -> Option<String> {
    let x1 = get_num(elem, "x1", 0.0);
    let y1 = get_num(elem, "y1", 0.0);
    let x2 = get_num(elem, "x2", 0.0);
    let y2 = get_num(elem, "y2", 0.0);

    Some(format!("M{} {}L{} {}", x1, y1, x2, y2))
}

fn convert_poly(elem: &Element, close: bool) -> Option<String> {
    let points = elem.attributes.get("points")?;
    let points_clean = points.replace(',', " ");
    let coords: Vec<&str> = points_clean.split_whitespace().collect();

    if coords.len() < 2 {
        return None;
    }

    let mut d = String::from("M");
    // This simple logic expects pairs.
    // real parser would be better but simple replace works for standard inputs

    // Actually, simple split might assume x y x y
    // We construct "M x1 y1 L x2 y2 L x3 y3"

    // Check if even?
    // Let's just append.
    // M x1 y1 L x2 y2 ...

    // First pair
    if coords.len() >= 2 {
        d.push_str(coords[0]);
        d.push(' ');
        d.push_str(coords[1]);

        // Rest
        for i in (2..coords.len()).step_by(2) {
            if i + 1 < coords.len() {
                d.push_str("L");
                d.push_str(coords[i]);
                d.push(' ');
                d.push_str(coords[i + 1]);
            }
        }
    }

    if close {
        d.push('z');
    }

    Some(d)
}

fn remove_shape_attrs(elem: &mut Element) {
    let attrs = [
        "x", "y", "width", "height", "rx", "ry", "r", "cx", "cy", "x1", "y1", "x2", "y2", "points",
    ];
    for attr in attrs {
        elem.attributes.shift_remove(attr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::printer;

    #[test]
    fn test_rect_to_path() {
        let input = "<svg><rect x=\"10\" y=\"20\" width=\"100\" height=\"50\"/></svg>";
        let expected = "<svg><path d=\"M10 20h100v50h-100z\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_keep_circle() {
        let input = "<svg><circle cx=\"50\" cy=\"50\" r=\"50\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_keep_ellipse() {
        let input = "<svg><ellipse cx=\"50\" cy=\"50\" rx=\"20\" ry=\"10\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_line_to_path() {
        let input = "<svg><line x1=\"10\" y1=\"10\" x2=\"50\" y2=\"50\"/></svg>";
        let expected = "<svg><path d=\"M10 10L50 50\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
    }

    #[test]
    fn test_polygon_to_path() {
        let input = "<svg><polygon points=\"0,0 100,0 100,100\"/></svg>";
        // M0 0L100 0L100 100z
        // Note: spaces handling
        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        let out = printer::print(&doc);
        assert!(out.contains("path"));
        assert!(out.contains("d=\"M0 0L100 0L100 100z"));
    }

    #[test]
    fn test_keep_rounded_rect_when_path_would_grow() {
        let input = "<svg><rect x=\"10\" y=\"20\" width=\"100\" height=\"50\" rx=\"8\" ry=\"6\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        assert_eq!(printer::print(&doc), input);
    }

    #[test]
    fn test_resolve_rect_radius_inherits_missing_axis() {
        assert_eq!(
            resolve_rect_radius(Some(8.0), None, 20.0, 10.0),
            Some((8.0, 5.0))
        );
    }
}
