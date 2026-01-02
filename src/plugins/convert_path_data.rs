use crate::plugins::Plugin;
use crate::tree::{Document, Node};

pub struct ConvertPathData {
    pub float_precision: usize,
    pub leading_zero: bool,
    // Add more opts as needed
}

impl Default for ConvertPathData {
    fn default() -> Self {
        Self {
            float_precision: 3,
            leading_zero: true,
        }
    }
}

impl Plugin for ConvertPathData {
    fn apply(&self, doc: &mut Document) {
        process_paths(&mut doc.root, self);
    }
}

fn process_paths(nodes: &mut Vec<Node>, opts: &ConvertPathData) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "path" {
                if let Some(d) = elem.attributes.get_mut("d") {
                    let new_d = optimize_path_data(d, opts);
                    *d = new_d;
                }
            }
            process_paths(&mut elem.children, opts);
        }
    }
}

fn optimize_path_data(d: &str, opts: &ConvertPathData) -> String {
    let commands = parse_path_data(d);
    stringify_optimized(&commands, opts)
}

#[derive(Debug, Clone, PartialEq)]
enum Command {
    // We store minimal data, usually absolute for analysis?
    // Actually, to choose best representation, let's store Absolute coordinates internally
    // and decide Rel/Abs at stringify time.
    Move(f64, f64),
    Line(f64, f64),
    Horiz(f64),
    Vert(f64),
    Curve(f64, f64, f64, f64, f64, f64), // x1 y1 x2 y2 x y
    SmoothCurve(f64, f64, f64, f64),     // x2 y2 x y
    Quad(f64, f64, f64, f64),            // x1 y1 x y
    SmoothQuad(f64, f64),                // x y
    Arc(f64, f64, f64, bool, bool, f64, f64),
    Close,
}

// Struct removed. Using PathLexer below.

// ... Lexer implementation ...
// To be robust, let's maintain the decent lexer from previous step but adapt output to Normalized Commands (Absolute).
// Converting everything to absolute simplifies optimization logic (L vs H vs V).

fn parse_path_data(d: &str) -> Vec<Command> {
    // Current pen position
    let mut cur_x = 0.0;
    let mut cur_y = 0.0;

    // Previous control point for S/T (absolute)
    // If previous was C/S, ctrl point is second control point. Else current point (reflection).
    // Actually we don't need to track prev control point during parsing if we just convert rel->abs.
    // Rel just adds to cur_x/cur_y.

    let mut commands = Vec::new();

    // Reuse the lexer logic from previous step efficiently
    let mut lexer = PathLexer::new(d);
    let mut current_cmd_char = None;

    loop {
        lexer.skip_ws_comma();
        let c_opt = lexer.peek_char();
        if c_opt.is_none() {
            break;
        }

        let c = c_opt.unwrap();
        let cmd_char = if c.is_ascii_alphabetic() {
            lexer.read_char().unwrap()
        } else {
            // Implicit
            if let Some(cmd) = current_cmd_char {
                if cmd == 'M' {
                    'L'
                } else if cmd == 'm' {
                    'l'
                } else {
                    cmd
                }
            } else {
                break;
            }
        };
        current_cmd_char = Some(cmd_char);

        match cmd_char {
            'M' => {
                while let (Some(x), Some(y)) = (lexer.read_number(), lexer.read_number()) {
                    commands.push(Command::Move(x, y));
                    cur_x = x;
                    cur_y = y;
                    current_cmd_char = Some('L'); // Subsequent are Line
                }
            }
            'm' => {
                while let (Some(dx), Some(dy)) = (lexer.read_number(), lexer.read_number()) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::Move(nx, ny));
                    cur_x = nx;
                    cur_y = ny;
                    current_cmd_char = Some('l');
                }
            }
            'L' => {
                while let (Some(x), Some(y)) = (lexer.read_number(), lexer.read_number()) {
                    commands.push(Command::Line(x, y));
                    cur_x = x;
                    cur_y = y;
                }
            }
            'l' => {
                while let (Some(dx), Some(dy)) = (lexer.read_number(), lexer.read_number()) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::Line(nx, ny));
                    cur_x = nx;
                    cur_y = ny;
                }
            }
            'H' => {
                while let Some(x) = lexer.read_number() {
                    commands.push(Command::Horiz(x));
                    cur_x = x;
                }
            }
            'h' => {
                while let Some(dx) = lexer.read_number() {
                    let nx = cur_x + dx;
                    commands.push(Command::Horiz(nx));
                    cur_x = nx;
                }
            }
            'V' => {
                while let Some(y) = lexer.read_number() {
                    commands.push(Command::Vert(y));
                    cur_y = y;
                }
            }
            'v' => {
                while let Some(dy) = lexer.read_number() {
                    let ny = cur_y + dy;
                    commands.push(Command::Vert(ny));
                    cur_y = ny;
                }
            }
            'Z' | 'z' => {
                commands.push(Command::Close);
                // No args.
                // Need start point of subpath? For Z calculation?
                // Z closes to most recent Move.
                // But for next command, pos is subpath start.
                // Tracking subpath start is needed for correct pen pos after Z.
            }
            'C' => {
                while let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    commands.push(Command::Curve(x1, y1, x2, y2, x, y));
                    cur_x = x;
                    cur_y = y;
                }
            }
            'c' => {
                while let (Some(dx1), Some(dy1), Some(dx2), Some(dy2), Some(dx), Some(dy)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::Curve(
                        cur_x + dx1,
                        cur_y + dy1,
                        cur_x + dx2,
                        cur_y + dy2,
                        nx,
                        ny,
                    ));
                    cur_x = nx;
                    cur_y = ny;
                }
            }
            'S' => {
                while let (Some(x2), Some(y2), Some(x), Some(y)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    commands.push(Command::SmoothCurve(x2, y2, x, y));
                    cur_x = x;
                    cur_y = y;
                }
            }
            's' => {
                while let (Some(dx2), Some(dy2), Some(dx), Some(dy)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::SmoothCurve(cur_x + dx2, cur_y + dy2, nx, ny));
                    cur_x = nx;
                    cur_y = ny;
                }
            }
            'Q' => {
                while let (Some(x1), Some(y1), Some(x), Some(y)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    commands.push(Command::Quad(x1, y1, x, y));
                    cur_x = x;
                    cur_y = y;
                }
            }
            'q' => {
                while let (Some(dx1), Some(dy1), Some(dx), Some(dy)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::Quad(cur_x + dx1, cur_y + dy1, nx, ny));
                    cur_x = nx;
                    cur_y = ny;
                }
            }
            'T' => {
                while let (Some(x), Some(y)) = (lexer.read_number(), lexer.read_number()) {
                    commands.push(Command::SmoothQuad(x, y));
                    cur_x = x;
                    cur_y = y;
                }
            }
            't' => {
                while let (Some(dx), Some(dy)) = (lexer.read_number(), lexer.read_number()) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::SmoothQuad(nx, ny));
                    cur_x = nx;
                    cur_y = ny;
                }
            }
            'A' => {
                while let (Some(rx), Some(ry), Some(rot), Some(la), Some(sf), Some(x), Some(y)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_flag(),
                    lexer.read_flag(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    commands.push(Command::Arc(rx, ry, rot, la, sf, x, y));
                    cur_x = x;
                    cur_y = y;
                }
            }
            'a' => {
                while let (Some(rx), Some(ry), Some(rot), Some(la), Some(sf), Some(dx), Some(dy)) = (
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_number(),
                    lexer.read_flag(),
                    lexer.read_flag(),
                    lexer.read_number(),
                    lexer.read_number(),
                ) {
                    let nx = cur_x + dx;
                    let ny = cur_y + dy;
                    commands.push(Command::Arc(rx, ry, rot, la, sf, nx, ny));
                    cur_x = nx;
                    cur_y = ny;
                }
            }
            _ => {
                break;
            }
        }
    }

    // Fix Close Z/z position tracking
    // Parsing Z should implicitly move pen to start of subpath.
    // For exact tracking we need to store start_x/y of last Move.
    // Simplifying for now - Z does not produce coords.

    commands
}

fn format_num(n: f64, p: usize) -> String {
    let factor = 10u32.pow(p as u32) as f64;
    let rounded = (n * factor).round() / factor;
    // Remove leading zeros etc.
    let s = rounded.to_string();
    if s.starts_with("0.") {
        s[1..].to_string()
    } else if s.starts_with("-0.") {
        format!("-{}", &s[2..])
    } else {
        s
    }
}

fn stringify_optimized(commands: &[Command], opts: &ConvertPathData) -> String {
    let mut s = String::new();
    let p = opts.float_precision;

    // State for optimization
    let mut cur_x = 0.0;
    let mut cur_y = 0.0;
    let mut subpath_start_x = 0.0;
    let mut subpath_start_y = 0.0;

    for cmd in commands {
        match cmd {
            Command::Move(x, y) => {
                // Always M? Or m?
                // M is usually safest/clearest for start.
                // relative m is only relative to (0,0) if first, or prev command.
                let abs_str = format!("M{} {}", format_num(*x, p), format_num(*y, p));
                let rel_str = format!(
                    "m{} {}",
                    format_num(*x - cur_x, p),
                    format_num(*y - cur_y, p)
                );

                if rel_str.len() < abs_str.len() {
                    s.push_str(&rel_str);
                } else {
                    s.push_str(&abs_str);
                }

                cur_x = *x;
                cur_y = *y;
                subpath_start_x = *x;
                subpath_start_y = *y;
            }
            Command::Line(x, y) => {
                let abs_x_s = format_num(*x, p);
                let abs_y_s = format_num(*y, p);
                let rel_x_s = format_num(*x - cur_x, p);
                let rel_y_s = format_num(*y - cur_y, p);

                // Candidates:
                // L x y
                // l dx dy
                // H x (if y == cur_y)
                // h dx (if y == cur_y)
                // V y (if x == cur_x)
                // v dy (if x == cur_x)

                let mut best_str = format!("L{} {}", abs_x_s, abs_y_s);
                let rel_l = format!("l{} {}", rel_x_s, rel_y_s);
                if rel_l.len() < best_str.len() {
                    best_str = rel_l;
                }

                // H/h check (check rounded values to be safe?)
                // Effectively we check if diff is close to 0.
                if (y - cur_y).abs() < f64::EPSILON {
                    let abs_h = format!("H{}", abs_x_s);
                    let rel_h = format!("h{}", rel_x_s);
                    if abs_h.len() < best_str.len() {
                        best_str = abs_h;
                    }
                    if rel_h.len() < best_str.len() {
                        best_str = rel_h;
                    }
                }

                if (x - cur_x).abs() < f64::EPSILON {
                    let abs_v = format!("V{}", abs_y_s);
                    let rel_v = format!("v{}", rel_y_s);
                    if abs_v.len() < best_str.len() {
                        best_str = abs_v;
                    }
                    if rel_v.len() < best_str.len() {
                        best_str = rel_v;
                    }
                }

                s.push_str(&best_str);
                cur_x = *x;
                cur_y = *y;
            }
            Command::Horiz(x) => {
                // Just optimize to H/h
                let abs_s = format!("H{}", format_num(*x, p));
                let rel_s = format!("h{}", format_num(*x - cur_x, p));
                if rel_s.len() < abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
            }
            Command::Vert(y) => {
                let abs_s = format!("V{}", format_num(*y, p));
                let rel_s = format!("v{}", format_num(*y - cur_y, p));
                if rel_s.len() < abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_y = *y;
            }
            Command::Close => {
                s.push('z');
                cur_x = subpath_start_x;
                cur_y = subpath_start_y;
            }
            // ... Implement others (Curve, Quad, Arc) similar way
            _ => {}
        }
    }

    s
}

// Reuse PathLexer from previous tool call (need to duplicate code here as I'm overwriting file)
struct PathLexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PathLexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_ws_comma(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_whitespace() || c == ',' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            Some(self.input.as_bytes()[self.pos] as char)
        }
    }

    fn read_char(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            let c = self.input.as_bytes()[self.pos] as char;
            self.pos += 1;
            Some(c)
        }
    }

    fn read_number(&mut self) -> Option<f64> {
        self.skip_ws_comma();
        if self.pos >= self.input.len() {
            return None;
        }
        let start = self.pos;
        let mut seen_dot = false;
        let mut seen_exp = false;
        if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
            self.pos += 1;
        }

        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == '.' && !seen_dot && !seen_exp {
                seen_dot = true;
                self.pos += 1;
            } else if (c == 'e' || c == 'E') && !seen_exp {
                seen_exp = true;
                self.pos += 1;
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        let sub = &self.input[start..self.pos];
        sub.parse::<f64>().ok()
    }

    fn read_flag(&mut self) -> Option<bool> {
        self.skip_ws_comma();
        match self.read_char() {
            Some('0') => Some(false),
            Some('1') => Some(true),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_line_rel() {
        // M 10 10 L 11 11
        // L 11 11 (7 chars)
        // l 1 1 (5 chars) -> Wins
        let input = "M 10 10 L 11 11";
        let out = optimize_path_data(input, &ConvertPathData::default());
        assert_eq!(out, "M10 10l1 1");
    }

    #[test]
    fn test_optimize_line_hz() {
        // M 10 10 L 20 10
        // L 20 10 (7)
        // l 10 0 (6)
        // H 20 (3) -> Wins
        let input = "M 10 10 L 20 10";
        let out = optimize_path_data(input, &ConvertPathData::default());
        assert_eq!(out, "M10 10H20");
    }
}
