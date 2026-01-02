use crate::plugins::Plugin;
use crate::tree::{Document, Node};
use std::f64::consts::PI;

pub struct ConvertTransform {
    pub float_precision: usize,
    pub deg_precision: usize,
}

impl Default for ConvertTransform {
    fn default() -> Self {
        Self {
            float_precision: 3,
            deg_precision: 3,
        }
    }
}

impl Plugin for ConvertTransform {
    fn apply(&self, doc: &mut Document) {
        process_transforms(&mut doc.root, self);
    }
}

fn process_transforms(nodes: &mut Vec<Node>, opts: &ConvertTransform) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if let Some(t) = elem.attributes.get_mut("transform") {
                let new_t = optimize_transform(t, opts);
                if new_t.is_empty() {
                    elem.attributes.shift_remove("transform"); // Remove if identity/empty
                } else {
                    *t = new_t;
                }
            }
            process_transforms(&mut elem.children, opts);
        }
    }
}

// 2D Affine Matrix
// [ a c e ]
// [ b d f ]
// [ 0 0 1 ]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Matrix {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Matrix {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn multiply(&self, other: &Matrix) -> Matrix {
        // Self * Other (Order depends on pre/post mult convention. SVG is post-mult: P' = M * P)
        Matrix {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }
}

fn optimize_transform(transform_str: &str, opts: &ConvertTransform) -> String {
    // 1. Parse into list of matrices
    let matrices = parse_transform(transform_str);
    if matrices.is_empty() {
        return String::new();
    }

    // 2. Multiply all into one
    let mut combined = Matrix::identity();
    for m in matrices {
        combined = combined.multiply(&m);
    }

    // 3. Decompose / Stringify
    // Check if identity
    if is_identity(&combined) {
        return String::new();
    }

    // Check for simple cases to stringify compactly
    // 1. Translate only (a=1, d=1, b=0, c=0)
    if is_approx(combined.a, 1.0)
        && is_approx(combined.d, 1.0)
        && is_approx(combined.b, 0.0)
        && is_approx(combined.c, 0.0)
    {
        return format!(
            "translate({} {})",
            format_num(combined.e, opts.float_precision),
            format_num(combined.f, opts.float_precision)
        );
    }

    // 2. Scale only (b=0, c=0, e=0, f=0)
    if is_approx(combined.b, 0.0)
        && is_approx(combined.c, 0.0)
        && is_approx(combined.e, 0.0)
        && is_approx(combined.f, 0.0)
    {
        if (combined.a - combined.d).abs() < f64::EPSILON {
            return format!("scale({})", format_num(combined.a, opts.float_precision));
        }
        return format!(
            "scale({} {})",
            format_num(combined.a, opts.float_precision),
            format_num(combined.d, opts.float_precision)
        );
    }

    // Default to matrix(...)
    format!(
        "matrix({} {} {} {} {} {})",
        format_num(combined.a, opts.float_precision),
        format_num(combined.b, opts.float_precision),
        format_num(combined.c, opts.float_precision),
        format_num(combined.d, opts.float_precision),
        format_num(combined.e, opts.float_precision),
        format_num(combined.f, opts.float_precision)
    )
}

fn is_approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}

fn is_identity(m: &Matrix) -> bool {
    is_approx(m.a, 1.0)
        && is_approx(m.b, 0.0)
        && is_approx(m.c, 0.0)
        && is_approx(m.d, 1.0)
        && is_approx(m.e, 0.0)
        && is_approx(m.f, 0.0)
}

fn format_num(n: f64, p: usize) -> String {
    let factor = 10u32.pow(p as u32) as f64;
    let rounded = (n * factor).round() / factor;
    let s = rounded.to_string();
    if s.starts_with("0.") {
        s[1..].to_string()
    } else if s.starts_with("-0.") {
        format!("-{}", &s[2..])
    } else {
        s
    }
}

fn parse_transform(s: &str) -> Vec<Matrix> {
    parse_transform_manual(s)
}

fn parse_transform_manual(s: &str) -> Vec<Matrix> {
    let mut matrices = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        if c.is_ascii_alphabetic() {
            // Read name
            let mut name = String::new();
            name.push(c);
            while let Some((_, nc)) = chars.peek() {
                if nc.is_ascii_alphabetic() {
                    name.push(*nc);
                    chars.next();
                } else {
                    break;
                }
            }

            // Skip to (
            while let Some((_, nc)) = chars.peek() {
                if *nc == '(' {
                    chars.next();
                    break;
                }
                chars.next();
            }

            // Read args
            let mut args = Vec::new();
            let mut cur_num = String::new();
            while let Some((_, nc)) = chars.peek() {
                if *nc == ')' {
                    chars.next();
                    break;
                }
                let cc = *nc;
                if cc.is_numeric() || cc == '.' || cc == '-' || cc == 'e' || cc == 'E' {
                    cur_num.push(cc);
                    chars.next();
                } else {
                    if !cur_num.is_empty() {
                        if let Ok(n) = cur_num.parse::<f64>() {
                            args.push(n);
                        }
                        cur_num.clear();
                    }
                    chars.next(); // Skip separator
                }
            }
            if !cur_num.is_empty() {
                if let Ok(n) = cur_num.parse::<f64>() {
                    args.push(n);
                }
            }

            match name.as_str() {
                "translate" => {
                    let tx = *args.get(0).unwrap_or(&0.0);
                    let ty = *args.get(1).unwrap_or(&0.0);
                    matrices.push(Matrix {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: tx,
                        f: ty,
                    });
                }
                "scale" => {
                    let sx = *args.get(0).unwrap_or(&1.0);
                    let sy = *args.get(1).unwrap_or(&sx); // if 1 arg, scale(s, s)
                    matrices.push(Matrix {
                        a: sx,
                        b: 0.0,
                        c: 0.0,
                        d: sy,
                        e: 0.0,
                        f: 0.0,
                    });
                }
                "rotate" => {
                    let angle = *args.get(0).unwrap_or(&0.0);
                    // cx, cy optional
                    let cx = *args.get(1).unwrap_or(&0.0);
                    let cy = *args.get(2).unwrap_or(&0.0);

                    let rad = angle * PI / 180.0;
                    let c = rad.cos();
                    let s = rad.sin();

                    // define rotate(a, cx, cy) as translate(cx, cy) rotate(a) translate(-cx, -cy)
                    let mut m = Matrix::identity();
                    if cx != 0.0 || cy != 0.0 {
                        m = m.multiply(&Matrix {
                            a: 1.0,
                            b: 0.0,
                            c: 0.0,
                            d: 1.0,
                            e: cx,
                            f: cy,
                        });
                    }
                    m = m.multiply(&Matrix {
                        a: c,
                        b: s,
                        c: -s,
                        d: c,
                        e: 0.0,
                        f: 0.0,
                    });
                    if cx != 0.0 || cy != 0.0 {
                        m = m.multiply(&Matrix {
                            a: 1.0,
                            b: 0.0,
                            c: 0.0,
                            d: 1.0,
                            e: -cx,
                            f: -cy,
                        });
                    }
                    matrices.push(m);
                }
                "skewX" => {
                    let a = *args.get(0).unwrap_or(&0.0);
                    let rad = a * PI / 180.0;
                    matrices.push(Matrix {
                        a: 1.0,
                        b: 0.0,
                        c: rad.tan(),
                        d: 1.0,
                        e: 0.0,
                        f: 0.0,
                    });
                }
                "skewY" => {
                    let a = *args.get(0).unwrap_or(&0.0);
                    let rad = a * PI / 180.0;
                    matrices.push(Matrix {
                        a: 1.0,
                        b: rad.tan(),
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                        f: 0.0,
                    });
                }
                "matrix" => {
                    if args.len() == 6 {
                        matrices.push(Matrix {
                            a: args[0],
                            b: args[1],
                            c: args[2],
                            d: args[3],
                            e: args[4],
                            f: args[5],
                        });
                    }
                }
                _ => {}
            }
        }
    }
    matrices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_merge() {
        // translate(10) translate(20) -> translate(30 0)
        let input = "translate(10) translate(20)";
        let opts = ConvertTransform::default();
        let out = optimize_transform(input, &opts);

        // translate(30 0) vs translate(30).
        assert!(out.contains("translate(30 0)"));
    }

    #[test]
    fn test_scale_merge() {
        let input = "scale(2) scale(3)";
        let opts = ConvertTransform::default();
        let out = optimize_transform(input, &opts);
        // 2*3 = 6
        assert!(out.contains("scale(6)"));
    }

    #[test]
    fn test_identity() {
        let input = "translate(0) scale(1)";
        let opts = ConvertTransform::default();
        let out = optimize_transform(input, &opts);
        assert_eq!(out, "");
    }
}
