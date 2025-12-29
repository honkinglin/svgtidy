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
        "circle" => convert_circle(elem),
        "ellipse" => convert_ellipse(elem),
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

    // Simplification: handle regular rects first
    // TODO: Handle rounded rects (rx, ry)
    if w == 0.0 || h == 0.0 {
        return None; // Invalid or invisible
    }

    if rx_raw.is_none() && ry_raw.is_none() {
        // M x y H x+w V y+h H x z
        // Or simpler: M x y h w v h h -w z
        // Let's use absolute for simplicity initially
        return Some(format!("M{} {}h{}v{}h-{}z", x, y, w, h, w));
    }

    // Rounded rects are complex to implement correctly with arcs.
    // For now, let's skip complex rounded rects or implement basic.
    None
}

fn convert_circle(elem: &Element) -> Option<String> {
    let r = get_num(elem, "r", 0.0);
    if r <= 0.0 {
        return None;
    }
    let cx = get_num(elem, "cx", 0.0);
    let cy = get_num(elem, "cy", 0.0);

    // Two Arcs
    // M cx-r cy A r r 0 1 0 cx+r cy A r r 0 1 0 cx-r cy
    Some(format!(
        "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}z",
        cx - r,
        cy,
        r,
        r,
        cx + r,
        cy,
        r,
        r,
        cx - r,
        cy
    ))
}

fn convert_ellipse(elem: &Element) -> Option<String> {
    let rx = get_num(elem, "rx", 0.0);
    let ry = get_num(elem, "ry", 0.0);
    if rx <= 0.0 || ry <= 0.0 {
        return None;
    }
    let cx = get_num(elem, "cx", 0.0);
    let cy = get_num(elem, "cy", 0.0);

    Some(format!(
        "M{} {}A{} {} 0 1 0 {} {}A{} {} 0 1 0 {} {}z",
        cx - rx,
        cy,
        rx,
        ry,
        cx + rx,
        cy,
        rx,
        ry,
        cx - rx,
        cy
    ))
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
    fn test_circle_to_path() {
        let input = "<svg><circle cx=\"50\" cy=\"50\" r=\"50\"/></svg>";
        // r=50, cx=50, cy=50.
        // M 0 50 A 50 50 0 1 0 100 50 A 50 50 0 1 0 0 50 z
        let expected = "<svg><path d=\"M0 50A50 50 0 1 0 100 50A50 50 0 1 0 0 50z\"/></svg>";

        let mut doc = parser::parse(input).unwrap();
        ConvertShapeToPath.apply(&mut doc);
        assert_eq!(printer::print(&doc), expected);
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
}
