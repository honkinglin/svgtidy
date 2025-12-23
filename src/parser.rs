use crate::tree::{Document, Element, Node};
use xmlparser::{Token, Tokenizer};

pub fn parse(text: &str) -> Result<Document, String> {
    let mut doc = Document::new();
    let mut element_stack: Vec<Element> = Vec::new();

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
                if let Some(current) = element_stack.last_mut() {
                    current.children.push(Node::Comment(content));
                } else {
                    doc.root.push(Node::Comment(content));
                }
            }
            Token::Cdata { text, .. } => {
                if let Some(current) = element_stack.last_mut() {
                    current
                        .children
                        .push(Node::Cdata(text.as_str().to_string()));
                }
            }
            Token::Declaration { .. } => {
                // xml declaration
                // For now might skip storing specific declaration info in AST,
                // or add a Node::Declaration
            }
            Token::ProcessingInstruction {
                target, content, ..
            } => {
                let t = target.as_str().to_string();
                let c = content.map(|s| s.as_str().to_string());
                if let Some(current) = element_stack.last_mut() {
                    current.children.push(Node::ProcessingInstruction(t, c));
                } else {
                    doc.root.push(Node::ProcessingInstruction(t, c));
                }
            }
            Token::DtdStart { .. }
            | Token::DtdEnd { .. }
            | Token::EmptyDtd { .. }
            | Token::EntityDeclaration { .. } => {
                // Simply ignore DTD for now or simple handling
            }
        }
    }

    Ok(doc)
}
