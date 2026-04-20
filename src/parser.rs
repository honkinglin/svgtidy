use crate::tree::{Document, Element, Node};
use xmlparser::{Token, Tokenizer};

pub fn parse(text: &str) -> Result<Document, String> {
    let mut doc = Document::new();
    let mut element_stack: Vec<Element> = Vec::new();
    let mut dtd_start: Option<usize> = None;

    for token in Tokenizer::from(text) {
        let token = token.map_err(|e| e.to_string())?;
        match token {
            Token::ElementStart { prefix, local, .. } => {
                let name = if prefix.is_empty() {
                    local.as_str().to_string()
                } else {
                    format!("{}:{}", prefix.as_str(), local.as_str())
                };
                let element = Element::new(name);
                element_stack.push(element);
            }
            Token::Attribute {
                prefix,
                local,
                value,
                ..
            } => {
                if let Some(current) = element_stack.last_mut() {
                    let key = if prefix.is_empty() {
                        local.as_str().to_string()
                    } else {
                        format!("{}:{}", prefix.as_str(), local.as_str())
                    };
                    current.attributes.insert(key, value.as_str().to_string());
                }
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    xmlparser::ElementEnd::Open => {
                        // Just finished attributes, nothing to do
                    }
                    xmlparser::ElementEnd::Close(..) | xmlparser::ElementEnd::Empty => {
                        if let Some(element) = element_stack.pop() {
                            if let Some(parent) = element_stack.last_mut() {
                                parent.children.push(Node::Element(element));
                            } else {
                                doc.root.push(Node::Element(element));
                            }
                        }
                    }
                }
            }
            Token::Text { text } => {
                let content = text.as_str().to_string();
                // Simple whitespace heuristic: if just whitespace, maybe ignore?
                // For now, keep everything to be safe.
                if let Some(current) = element_stack.last_mut() {
                    current.children.push(Node::Text(content));
                } else {
                    // Top level text? usually whitespace
                    // doc.root.push(Node::Text(content));
                }
            }
            Token::Comment { text, .. } => {
                let content = text.as_str().to_string();
                push_node(&mut doc, &mut element_stack, Node::Comment(content));
            }
            Token::Cdata { text, .. } => {
                push_node(
                    &mut doc,
                    &mut element_stack,
                    Node::Cdata(text.as_str().to_string()),
                );
            }
            Token::Declaration { span, .. } => {
                let raw = span.as_str();
                let content = raw
                    .strip_prefix("<?xml")
                    .and_then(|value| value.strip_suffix("?>"))
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                push_node(
                    &mut doc,
                    &mut element_stack,
                    Node::ProcessingInstruction("xml".to_string(), content),
                );
            }
            Token::ProcessingInstruction {
                target, content, ..
            } => {
                let t = target.as_str().to_string();
                let c = content.map(|s| s.as_str().to_string());
                push_node(
                    &mut doc,
                    &mut element_stack,
                    Node::ProcessingInstruction(t, c),
                );
            }
            Token::DtdStart { span, .. } => {
                dtd_start = Some(span.start());
            }
            Token::DtdEnd { span } => {
                if let Some(start) = dtd_start.take() {
                    let raw = &text[start..span.end()];
                    if let Some(content) = doctype_content(raw) {
                        push_node(&mut doc, &mut element_stack, Node::Doctype(content));
                    }
                }
            }
            Token::EmptyDtd { span, .. } => {
                if let Some(content) = doctype_content(span.as_str()) {
                    push_node(&mut doc, &mut element_stack, Node::Doctype(content));
                }
            }
            Token::EntityDeclaration { .. } => {}
        }
    }

    Ok(doc)
}

fn push_node(doc: &mut Document, element_stack: &mut [Element], node: Node) {
    if let Some(current) = element_stack.last_mut() {
        current.children.push(node);
    } else {
        doc.root.push(node);
    }
}

fn doctype_content(raw: &str) -> Option<String> {
    raw.strip_prefix("<!DOCTYPE")
        .and_then(|value| value.strip_suffix('>'))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printer;

    #[test]
    fn test_parse_preserves_xml_declaration() {
        let input = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg></svg>";
        let doc = parse(input).unwrap();

        assert_eq!(
            printer::print(&doc),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg/>"
        );
    }

    #[test]
    fn test_parse_preserves_doctype() {
        let input =
            "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\"><svg></svg>";
        let doc = parse(input).unwrap();

        assert_eq!(
            printer::print(&doc),
            "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\"><svg/>"
        );
    }
}
