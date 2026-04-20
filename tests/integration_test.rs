use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use svgtidy::parser;
use svgtidy::pipeline::OptimizeOptions;
use svgtidy::{optimize, optimize_with_options};

#[test]
fn test_svg_cases() {
    let entries = svg_case_paths();
    let mut analyzed = 0;

    for entry in entries {
        let filename = entry.file_name().unwrap().to_string_lossy();
        println!("Testing {}", filename);

        let input = fs::read_to_string(&entry).expect("Failed to read SVG file");
        let output = optimize(&input);
        let reparsed = parser::parse(&output).unwrap_or_else(|err| {
            panic!("Optimized output for {} is not parseable: {err}", filename)
        });

        assert!(
            has_root_svg(&reparsed),
            "Output for {} should still contain a root <svg> element",
            filename
        );
        assert!(
            output.len() <= input.len(),
            "Output for {} should be smaller or equal to input",
            filename
        );
        assert_eq!(
            optimize(&output),
            output,
            "Optimization for {} should be idempotent",
            filename
        );

        analyzed += 1;
    }

    assert!(analyzed > 0, "No SVG test cases were found!");
}

#[test]
fn test_default_pipeline_removes_root_level_nodes() {
    let input =
        "<?xml version=\"1.0\"?><!DOCTYPE svg><svg><!--comment--><rect width=\"0\" height=\"10\"/></svg>";

    assert_eq!(optimize(input), "<svg/>");
}

#[test]
fn test_disable_root_level_cleanup_plugins_preserves_nodes() {
    let input = "<?xml version=\"1.0\"?><!DOCTYPE svg><svg><!--comment--></svg>";
    let mut options = OptimizeOptions::default();
    options.disable.insert("removeDoctype".to_string());
    options.disable.insert("removeXMLProcInst".to_string());
    options.disable.insert("removeComments".to_string());

    let output = optimize_with_options(input, &options).unwrap();

    assert_eq!(
        output,
        "<?xml version=\"1.0\"?><!DOCTYPE svg><svg><!--comment--></svg>"
    );
}

fn svg_case_paths() -> Vec<PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_cases_dir = Path::new(&manifest_dir).join("test-cases");

    if !test_cases_dir.exists() {
        panic!("Test cases directory not found at {:?}", test_cases_dir);
    }

    let mut entries = fs::read_dir(&test_cases_dir)
        .expect("Failed to read test-cases directory")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .expect("Failed to process directory entries");

    entries.retain(|path| path.extension().and_then(|s| s.to_str()) == Some("svg"));
    entries.sort();
    entries
}

fn has_root_svg(doc: &svgtidy::tree::Document) -> bool {
    doc.root.iter().any(|node| {
        matches!(
            node,
            svgtidy::tree::Node::Element(element) if element.name == "svg"
        )
    })
}
