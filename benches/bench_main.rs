use criterion::{black_box, criterion_group, criterion_main, Criterion};
use svgtidy::parser;
use svgtidy::plugins::{
    CleanupAttrs, CleanupIds, CleanupListOfValues, CleanupNumericValues, CollapseGroups,
    ConvertColors, ConvertEllipseToCircle, ConvertOneStopGradients, ConvertPathData,
    ConvertShapeToPath, ConvertStyleToAttrs, ConvertTransform, MergePaths, MoveElemsAttrsToGroup,
    MoveGroupAttrsToElems, Plugin, RemoveComments, RemoveDesc, RemoveDimensions, RemoveDoctype,
    RemoveEditorsNSData, RemoveEmptyAttrs, RemoveEmptyContainers, RemoveEmptyText,
    RemoveHiddenElems, RemoveMetadata, RemoveRasterImages, RemoveScriptElement, RemoveStyleElement,
    RemoveTitle, RemoveUnknownsAndDefaults, RemoveUnusedNS, RemoveUselessDefs,
    RemoveUselessStrokeAndFill, RemoveXMLProcInst, SortAttrs, SortDefsChildren,
};
use svgtidy::printer;

fn get_complex_svg() -> String {
    let mut s =
        String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 1000\">");
    // Add some random paths and groups
    for i in 0..100 {
        s.push_str(&format!("<g transform=\"translate({}, {})\">", i, i));
        s.push_str(&format!("<rect x=\"10\" y=\"10\" width=\"100\" height=\"100\" fill=\"#ff0000\" stroke=\"blue\" stroke-width=\"{}\"/>", i % 5));
        s.push_str("<path d=\"M10 10 L 20 20 C 30 30, 40 40, 50 50 Z\"/>");
        s.push_str("</g>");
    }
    s.push_str("</svg>");
    s
}

fn get_icon_svg() -> String {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="12" y1="8" x2="12" y2="12"></line>
        <line x1="12" y1="16" x2="12.01" y2="16"></line>
        <desc>Icon Description</desc>
        <title>Icon Title</title>
    </svg>"#.to_string()
}

fn run_pipeline(input: &str) -> String {
    let mut doc = parser::parse(input).unwrap();

    // Replicating main.rs pipeline
    let plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(RemoveDoctype),
        Box::new(RemoveXMLProcInst),
        Box::new(RemoveComments),
        Box::new(RemoveMetadata),
        Box::new(RemoveTitle),
        Box::new(RemoveDesc),
        Box::new(RemoveEditorsNSData),
        Box::new(RemoveScriptElement),
        Box::new(RemoveRasterImages),
        Box::new(RemoveStyleElement),
        Box::new(ConvertStyleToAttrs),
        Box::new(CleanupAttrs),
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
        Box::new(RemoveUselessStrokeAndFill),
        Box::new(ConvertEllipseToCircle),
        Box::new(ConvertShapeToPath),
        Box::new(ConvertPathData::default()),
        Box::new(ConvertTransform::default()),
        Box::new(MergePaths),
        Box::new(ConvertColors),
        Box::new(CleanupNumericValues::default()),
        Box::new(CleanupListOfValues::default()),
        Box::new(RemoveUnknownsAndDefaults::default()),
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

fn bench_parser(c: &mut Criterion) {
    let icon = get_icon_svg();
    let complex = get_complex_svg();

    let mut group = c.benchmark_group("Parser");
    group.bench_function("parse_icon", |b| b.iter(|| parser::parse(black_box(&icon))));
    group.bench_function("parse_complex", |b| {
        b.iter(|| parser::parse(black_box(&complex)))
    });
    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let icon = get_icon_svg();
    let complex = get_complex_svg();

    let mut group = c.benchmark_group("Full Pipeline");
    group.bench_function("optimize_icon", |b| {
        b.iter(|| run_pipeline(black_box(&icon)))
    });
    group.bench_function("optimize_complex", |b| {
        b.iter(|| run_pipeline(black_box(&complex)))
    });
    group.finish();
}

criterion_group!(benches, bench_parser, bench_full_pipeline);
criterion_main!(benches);
