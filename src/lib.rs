pub mod parser;
pub mod pipeline;
pub mod plugins;
pub mod printer;
pub mod tree;
pub mod visitor;

use crate::pipeline::{apply_default_pipeline, OptimizeOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn optimize(svg: &str) -> String {
    match optimize_with_options(svg, &OptimizeOptions::default()) {
        Ok(output) => output,
        Err(_) => svg.to_string(),
    }
}

pub fn optimize_with_options(svg: &str, options: &OptimizeOptions) -> Result<String, String> {
    let mut doc = match parser::parse(svg) {
        Ok(doc) => doc,
        Err(error) => return Err(error),
    };

    apply_default_pipeline(&mut doc, options);
    Ok(printer::print(&doc))
}
