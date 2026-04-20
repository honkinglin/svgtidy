use crate::tree::{Document, Node};

pub fn print(doc: &Document) -> String {
    let mut out = String::new();
    for node in &doc.root {
        print_node(node, &mut out);
    }
    out
}

pub fn print_pretty(doc: &Document) -> String {
    let mut out = String::new();
    for node in &doc.root {
        print_node_pretty(node, &mut out, 0);
    }
    out
}

fn print_node(node: &Node, out: &mut String) {
    match node {
        Node::Element(elem) => {
            out.push('<');
            out.push_str(&elem.name);
            for (k, v) in &elem.attributes {
                out.push(' ');
                out.push_str(k);
                out.push_str("=\"");
                out.push_str(v);
                out.push('"');
            }

            if elem.children.is_empty() {
                out.push_str("/>");
            } else {
                out.push('>');
                for child in &elem.children {
                    print_node(child, out);
                }
                out.push_str("</");
                out.push_str(&elem.name);
                out.push('>');
            }
        }
        Node::Text(text) => {
            out.push_str(text);
        }
        Node::Comment(text) => {
            out.push_str("<!--");
            out.push_str(text);
            out.push_str("-->");
        }
        Node::Cdata(text) => {
            out.push_str("<![CDATA[");
            out.push_str(text);
            out.push_str("]]>");
        }
        Node::ProcessingInstruction(target, content) => {
            out.push_str("<?");
            out.push_str(target);
            if let Some(c) = content {
                out.push(' ');
                out.push_str(c);
            }
            out.push_str("?>");
        }
        Node::Doctype(text) => {
            out.push_str("<!DOCTYPE ");
            out.push_str(text);
            out.push_str(">");
        }
    }
}

fn print_node_pretty(node: &Node, out: &mut String, indent: usize) {
    match node {
        Node::Element(elem) => {
            write_indent(out, indent);
            out.push('<');
            out.push_str(&elem.name);
            for (k, v) in &elem.attributes {
                out.push(' ');
                out.push_str(k);
                out.push_str("=\"");
                out.push_str(v);
                out.push('"');
            }

            if elem.children.is_empty() {
                out.push_str("/>\n");
                return;
            }

            if has_only_text_children(&elem.children) {
                out.push('>');
                for child in &elem.children {
                    print_node(child, out);
                }
                out.push_str("</");
                out.push_str(&elem.name);
                out.push_str(">\n");
                return;
            }

            out.push_str(">\n");
            for child in &elem.children {
                print_node_pretty(child, out, indent + 1);
            }
            write_indent(out, indent);
            out.push_str("</");
            out.push_str(&elem.name);
            out.push_str(">\n");
        }
        Node::Text(text) => {
            write_indent(out, indent);
            out.push_str(text);
            out.push('\n');
        }
        Node::Comment(text) => {
            write_indent(out, indent);
            out.push_str("<!--");
            out.push_str(text);
            out.push_str("-->\n");
        }
        Node::Cdata(text) => {
            write_indent(out, indent);
            out.push_str("<![CDATA[");
            out.push_str(text);
            out.push_str("]]>\n");
        }
        Node::ProcessingInstruction(target, content) => {
            write_indent(out, indent);
            out.push_str("<?");
            out.push_str(target);
            if let Some(c) = content {
                out.push(' ');
                out.push_str(c);
            }
            out.push_str("?>\n");
        }
        Node::Doctype(text) => {
            write_indent(out, indent);
            out.push_str("<!DOCTYPE ");
            out.push_str(text);
            out.push_str(">\n");
        }
    }
}

fn has_only_text_children(children: &[Node]) -> bool {
    children.iter().all(|child| matches!(child, Node::Text(_)))
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_print_pretty_nested_elements() {
        let input = "<svg><g><rect width=\"10\"/></g></svg>";
        let doc = parser::parse(input).unwrap();

        let expected = "<svg>\n  <g>\n    <rect width=\"10\"/>\n  </g>\n</svg>\n";
        assert_eq!(print_pretty(&doc), expected);
    }

    #[test]
    fn test_print_pretty_keeps_text_inline() {
        let input = "<svg><text>Hello</text></svg>";
        let doc = parser::parse(input).unwrap();

        let expected = "<svg>\n  <text>Hello</text>\n</svg>\n";
        assert_eq!(print_pretty(&doc), expected);
    }
}
