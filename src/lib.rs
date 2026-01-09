pub mod parser;
pub mod plugins;
pub mod printer;
pub mod tree;
pub mod visitor;

use crate::plugins::{
    CleanupAttrs, CleanupIds, CleanupListOfValues, CleanupNumericValues, CollapseGroups,
    ConvertColors, ConvertEllipseToCircle, ConvertOneStopGradients, ConvertPathData,
    ConvertShapeToPath, ConvertStyleToAttrs, ConvertTransform, MergePaths, MoveElemsAttrsToGroup,
    MoveGroupAttrsToElems, Plugin, RemoveComments, RemoveDesc, RemoveDimensions, RemoveDoctype,
    RemoveEditorsNSData, RemoveEmptyAttrs, RemoveEmptyContainers, RemoveEmptyText,
    RemoveHiddenElems, RemoveMetadata, RemoveRasterImages, RemoveScriptElement, RemoveStyleElement,
    RemoveTitle, RemoveUnknownsAndDefaults, RemoveUnusedNS, RemoveUselessDefs,
    RemoveUselessStrokeAndFill, RemoveXMLProcInst, SortAttrs, SortDefsChildren,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn optimize(svg: &str) -> String {
    let mut doc = match parser::parse(svg) {
        Ok(d) => d,
        Err(_) => return svg.to_string(), // Return original on error for safe WASM usage
    };

    // Default Pipeline
    let precision = 3;

    // Helper to create plugin (simplified vs main.rs)
    // We just instantiate directly here.
    let mut plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(RemoveDoctype),
        Box::new(RemoveXMLProcInst),
        Box::new(RemoveComments),
        Box::new(RemoveMetadata),
        Box::new(RemoveTitle),
        Box::new(RemoveDesc),
        Box::new(RemoveEditorsNSData),
        Box::new(RemoveScriptElement),
        Box::new(RemoveRasterImages),
        // RemoveStyleElement false by default in main, let's keep it out or minimal
        Box::new(ConvertStyleToAttrs),
        Box::new(CleanupAttrs),
        Box::new(RemoveUselessStrokeAndFill),
        Box::new(RemoveDimensions),
        Box::new(MoveGroupAttrsToElems),
        Box::new(MoveElemsAttrsToGroup),
        Box::new(ConvertOneStopGradients),
        Box::new(CleanupIds),
        Box::new(RemoveUselessDefs),
        Box::new(RemoveEmptyContainers),
        Box::new(RemoveHiddenElems),
        Box::new(RemoveEmptyText),
        Box::new(CollapseGroups),
        Box::new(ConvertEllipseToCircle),
        Box::new(ConvertShapeToPath),
        Box::new(ConvertPathData {
            float_precision: precision,
            leading_zero: true,
            ..Default::default()
        }),
        Box::new(ConvertTransform {
            float_precision: precision,
            deg_precision: precision,
            ..Default::default()
        }),
        Box::new(CleanupNumericValues {
            float_precision: precision,
            remove_px: true,
            leading_zero: true,
        }),
        Box::new(CleanupListOfValues {
            float_precision: precision,
            default_px: true,
            convert_to_px: true,
            leading_zero: true,
        }),
        Box::new(RemoveUnknownsAndDefaults::default()),
        Box::new(MergePaths),
        Box::new(ConvertColors),
        Box::new(RemoveEmptyAttrs),
        Box::new(RemoveUnusedNS),
        Box::new(SortAttrs),
        Box::new(SortDefsChildren),
    ];

    for plugin in plugins {
        plugin.apply(&mut doc);
    }

    printer::print(&doc)
}
