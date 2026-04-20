pub mod parser;
pub mod pipeline;
pub mod plugins;
pub mod printer;
pub mod tree;
pub mod visitor;

use crate::pipeline::{apply_default_pipeline, OptimizeOptions};
use crate::tree::Document;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn optimize(svg: &str) -> String {
    match optimize_with_options(svg, &OptimizeOptions::default()) {
        Ok(output) => output,
        Err(_) => svg.to_string(),
    }
}

pub fn optimize_with_options(svg: &str, options: &OptimizeOptions) -> Result<String, String> {
    let doc = optimize_to_document(svg, options)?;
    Ok(printer::print(&doc))
}

pub fn optimize_to_document(svg: &str, options: &OptimizeOptions) -> Result<Document, String> {
    let mut doc = parser::parse(svg)?;
    apply_default_pipeline(&mut doc, options);
    Ok(doc)
}
