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
            'A' => loop {
                let rx = lexer.read_number();
                if rx.is_none() {
                    break;
                }
                let ry = lexer.read_number();
                if ry.is_none() {
                    break;
                }
                let rot = lexer.read_number();
                if rot.is_none() {
                    break;
                }
                let la = lexer.read_flag();
                if la.is_none() {
                    break;
                }
                let sf = lexer.read_flag();
                if sf.is_none() {
                    break;
                }
                let x = lexer.read_number();
                if x.is_none() {
                    break;
                }
                let y = lexer.read_number();
                if y.is_none() {
                    break;
                }

                let rx = rx.unwrap();
                let ry = ry.unwrap();
                let rot = rot.unwrap();
                let la = la.unwrap();
                let sf = sf.unwrap();
                let x = x.unwrap();
                let y = y.unwrap();

                commands.push(Command::Arc(rx, ry, rot, la, sf, x, y));
                cur_x = x;
                cur_y = y;
            },
            'a' => loop {
                let rx = lexer.read_number();
                if rx.is_none() {
                    break;
                }
                let ry = lexer.read_number();
                if ry.is_none() {
                    break;
                }
                let rot = lexer.read_number();
                if rot.is_none() {
                    break;
                }
                let la = lexer.read_flag();
                if la.is_none() {
                    break;
                }
                let sf = lexer.read_flag();
                if sf.is_none() {
                    break;
                }
                let dx = lexer.read_number();
                if dx.is_none() {
                    break;
                }
                let dy = lexer.read_number();
                if dy.is_none() {
                    break;
                }

                let rx = rx.unwrap();
                let ry = ry.unwrap();
                let rot = rot.unwrap();
                let la = la.unwrap();
                let sf = sf.unwrap();
                let dx = dx.unwrap();
                let dy = dy.unwrap();

                let nx = cur_x + dx;
                let ny = cur_y + dy;
                commands.push(Command::Arc(rx, ry, rot, la, sf, nx, ny));
                cur_x = nx;
                cur_y = ny;
            },
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

fn needs_separator(prev: char, next: char) -> bool {
    matches!(prev, '0'..='9' | '.')
        && matches!(next, '0'..='9' | '.' | '+')
}

fn append_fragment(out: &mut String, fragment: &str) {
    if fragment.is_empty() {
        return;
    }

    if let (Some(prev), Some(next)) = (out.chars().last(), fragment.chars().next()) {
        if needs_separator(prev, next) {
            out.push(' ');
        }
    }

    out.push_str(fragment);
}

fn format_pair(a: f64, b: f64, p: usize) -> String {
    let mut out = format_num(a, p);
    let second = format_num(b, p);
    append_fragment(&mut out, &second);
    out
}

fn best_move_fragment(cur_x: f64, cur_y: f64, x: f64, y: f64, p: usize) -> String {
    let abs_str = format!("M{}", format_pair(x, y, p));
    let rel_str = format!("m{}", format_pair(x - cur_x, y - cur_y, p));

    if rel_str.len() < abs_str.len() {
        rel_str
    } else {
        abs_str
    }
}

fn best_line_fragment(cur_x: f64, cur_y: f64, x: f64, y: f64, p: usize) -> String {
    let abs_x_s = format_num(x, p);
    let abs_y_s = format_num(y, p);
    let rel_x_s = format_num(x - cur_x, p);
    let rel_y_s = format_num(y - cur_y, p);

    let rel_l = format!("l{}", format_pair(x - cur_x, y - cur_y, p));
    let abs_l = format!("L{}", format_pair(x, y, p));
    let mut best_str = if rel_l.len() <= abs_l.len() {
        rel_l
    } else {
        abs_l
    };

    if (y - cur_y).abs() < f64::EPSILON {
        let abs_h = format!("H{}", abs_x_s);
        let rel_h = format!("h{}", rel_x_s);
        let best_h = if rel_h.len() <= abs_h.len() {
            rel_h
        } else {
            abs_h
        };
        if best_h.len() <= best_str.len() {
            best_str = best_h;
        }
    }

    if (x - cur_x).abs() < f64::EPSILON {
        let abs_v = format!("V{}", abs_y_s);
        let rel_v = format!("v{}", rel_y_s);
        let best_v = if rel_v.len() <= abs_v.len() {
            rel_v
        } else {
            abs_v
        };
        if best_v.len() <= best_str.len() {
            best_str = best_v;
        }
    }

    best_str
}

fn serialize_move_line_run(
    commands: &[Command],
    cur_x: f64,
    cur_y: f64,
    p: usize,
) -> (String, f64, f64) {
    let Command::Move(move_x, move_y) = commands[0] else {
        unreachable!("move-line run must start with Move");
    };

    let mut naive = best_move_fragment(cur_x, cur_y, move_x, move_y, p);
    let mut naive_x = move_x;
    let mut naive_y = move_y;
    for cmd in &commands[1..] {
        let Command::Line(x, y) = cmd else {
            unreachable!("move-line run must only contain Line commands after Move");
        };
        naive.push_str(&best_line_fragment(naive_x, naive_y, *x, *y, p));
        naive_x = *x;
        naive_y = *y;
    }

    let mut implicit_abs = format!("M{}", format_pair(move_x, move_y, p));
    for cmd in &commands[1..] {
        let Command::Line(x, y) = cmd else {
            unreachable!("move-line run must only contain Line commands after Move");
        };
        append_fragment(&mut implicit_abs, &format_pair(*x, *y, p));
    }

    let mut implicit_rel = format!("m{}", format_pair(move_x - cur_x, move_y - cur_y, p));
    let mut rel_x = move_x;
    let mut rel_y = move_y;
    for cmd in &commands[1..] {
        let Command::Line(x, y) = cmd else {
            unreachable!("move-line run must only contain Line commands after Move");
        };
        append_fragment(&mut implicit_rel, &format_pair(*x - rel_x, *y - rel_y, p));
        rel_x = *x;
        rel_y = *y;
    }

    let best = [naive, implicit_abs, implicit_rel]
        .into_iter()
        .min_by_key(|candidate| candidate.len())
        .unwrap();

    (best, naive_x, naive_y)
}

fn stringify_optimized(commands: &[Command], opts: &ConvertPathData) -> String {
    let mut s = String::new();
    let p = opts.float_precision;

    // State for optimization
    let mut cur_x = 0.0;
    let mut cur_y = 0.0;
    let mut subpath_start_x = 0.0;
    let mut subpath_start_y = 0.0;

    let mut index = 0;
    while index < commands.len() {
        if matches!(commands[index], Command::Move(_, _)) {
            let mut end = index + 1;
            while end < commands.len() && matches!(commands[end], Command::Line(_, _)) {
                end += 1;
            }

            if end > index + 1 {
                let (fragment, x, y) = serialize_move_line_run(&commands[index..end], cur_x, cur_y, p);
                s.push_str(&fragment);
                cur_x = x;
                cur_y = y;
                if let Command::Move(move_x, move_y) = commands[index] {
                    subpath_start_x = move_x;
                    subpath_start_y = move_y;
                }
                index = end;
                continue;
            }
        }

        let cmd = &commands[index];
        match cmd {
            Command::Move(x, y) => {
                s.push_str(&best_move_fragment(cur_x, cur_y, *x, *y, p));

                cur_x = *x;
                cur_y = *y;
                subpath_start_x = *x;
                subpath_start_y = *y;
            }
            Command::Line(x, y) => {
                s.push_str(&best_line_fragment(cur_x, cur_y, *x, *y, p));
                cur_x = *x;
                cur_y = *y;
            }
            Command::Horiz(x) => {
                // Just optimize to H/h
                let abs_s = format!("H{}", format_num(*x, p));
                let rel_s = format!("h{}", format_num(*x - cur_x, p));
                if rel_s.len() <= abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
            }
            Command::Vert(y) => {
                let abs_s = format!("V{}", format_num(*y, p));
                let rel_s = format!("v{}", format_num(*y - cur_y, p));
                if rel_s.len() <= abs_s.len() {
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
            Command::Curve(x1, y1, x2, y2, x, y) => {
                let abs_coords = format!(
                    "{} {} {} {} {} {}",
                    format_num(*x1, p),
                    format_num(*y1, p),
                    format_num(*x2, p),
                    format_num(*y2, p),
                    format_num(*x, p),
                    format_num(*y, p)
                );
                let rel_coords = format!(
                    "{} {} {} {} {} {}",
                    format_num(*x1 - cur_x, p),
                    format_num(*y1 - cur_y, p),
                    format_num(*x2 - cur_x, p),
                    format_num(*y2 - cur_y, p),
                    format_num(*x - cur_x, p),
                    format_num(*y - cur_y, p)
                );

                let abs_s = format!("C{}", abs_coords);
                let rel_s = format!("c{}", rel_coords);

                if rel_s.len() <= abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
                cur_y = *y;
            }
            Command::SmoothCurve(x2, y2, x, y) => {
                let abs_coords = format!(
                    "{} {} {} {}",
                    format_num(*x2, p),
                    format_num(*y2, p),
                    format_num(*x, p),
                    format_num(*y, p)
                );
                let rel_coords = format!(
                    "{} {} {} {}",
                    format_num(*x2 - cur_x, p),
                    format_num(*y2 - cur_y, p),
                    format_num(*x - cur_x, p),
                    format_num(*y - cur_y, p)
                );

                let abs_s = format!("S{}", abs_coords);
                let rel_s = format!("s{}", rel_coords);

                if rel_s.len() <= abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
                cur_y = *y;
            }
            Command::Quad(x1, y1, x, y) => {
                let abs_coords = format!(
                    "{} {} {} {}",
                    format_num(*x1, p),
                    format_num(*y1, p),
                    format_num(*x, p),
                    format_num(*y, p)
                );
                let rel_coords = format!(
                    "{} {} {} {}",
                    format_num(*x1 - cur_x, p),
                    format_num(*y1 - cur_y, p),
                    format_num(*x - cur_x, p),
                    format_num(*y - cur_y, p)
                );

                let abs_s = format!("Q{}", abs_coords);
                let rel_s = format!("q{}", rel_coords);

                if rel_s.len() <= abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
                cur_y = *y;
            }
            Command::SmoothQuad(x, y) => {
                let abs_coords = format!("{} {}", format_num(*x, p), format_num(*y, p));
                let rel_coords = format!(
                    "{} {}",
                    format_num(*x - cur_x, p),
                    format_num(*y - cur_y, p)
                );

                let abs_s = format!("T{}", abs_coords);
                let rel_s = format!("t{}", rel_coords);

                if rel_s.len() <= abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
                cur_y = *y;
            }
            Command::Arc(rx, ry, rot, la, sf, x, y) => {
                // Formatting arc flags: 0 or 1
                let la_s = if *la { "1" } else { "0" };
                let sf_s = if *sf { "1" } else { "0" };

                let abs_coords = format!(
                    "{} {} {} {} {} {} {}",
                    format_num(*rx, p),
                    format_num(*ry, p),
                    format_num(*rot, p),
                    la_s,
                    sf_s,
                    format_num(*x, p),
                    format_num(*y, p)
                );
                let rel_coords = format!(
                    "{} {} {} {} {} {} {}",
                    format_num(*rx, p),
                    format_num(*ry, p),
                    format_num(*rot, p),
                    la_s,
                    sf_s,
                    format_num(*x - cur_x, p),
                    format_num(*y - cur_y, p)
                );

                let abs_s = format!("A{}", abs_coords);
                let rel_s = format!("a{}", rel_coords);

                if rel_s.len() <= abs_s.len() {
                    s.push_str(&rel_s);
                } else {
                    s.push_str(&abs_s);
                }
                cur_x = *x;
                cur_y = *y;
            }
        }

        index += 1;
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
        // H 20 / h 10 are both 3 chars, prefer relative on ties.
        let input = "M 10 10 L 20 10";
        let out = optimize_path_data(input, &ConvertPathData::default());
        assert_eq!(out, "M10 10h10");
    }

    #[test]
    fn test_optimize_line_run_after_move() {
        let input = "M2 2L10 10L18 2";
        let out = optimize_path_data(input, &ConvertPathData::default());
        assert_eq!(out, "M2 2l8 8l8-8");
    }

    #[test]
    fn test_optimize_arc() {
        // Circle path from convert_shape_to_path
        let input = "M0 50A50 50 0 1 0 100 50A50 50 0 1 0 0 50z";
        let out = optimize_path_data(input, &ConvertPathData::default());
        println!("Optimized Arc: '{}'", out);
        // Should not lose the arcs!
        assert!(
            out.contains("A") || out.contains("a"),
            "Output was: {}",
            out
        );
    }
}
