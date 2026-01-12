use std::env;
use std::fs;
use std::path::Path;
use svgtidy::optimize;

#[test]
fn test_svg_cases() {
    // Navigate to the test-cases directory at the project root
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_cases_dir = Path::new(&manifest_dir).join("test-cases");

    if !test_cases_dir.exists() {
        panic!("Test cases directory not found at {:?}", test_cases_dir);
    }

    let entries = fs::read_dir(&test_cases_dir)
        .expect("Failed to read test-cases directory")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .expect("Failed to process directory entries");

    let mut analyzed = 0;

    for entry in entries {
        if entry.extension().and_then(|s| s.to_str()) == Some("svg") {
            let filename = entry.file_name().unwrap().to_string_lossy();
            println!("Testing {}", filename);

            let input = fs::read_to_string(&entry).expect("Failed to read SVG file");
            let output = optimize(&input);

            assert!(
                output.starts_with("<svg"),
                "Output for {} should start with <svg",
                filename
            );
            assert!(
                output.len() <= input.len(),
                "Output for {} should be smaller or equal to input",
                filename
            );

            analyzed += 1;
        }
    }

    assert!(analyzed > 0, "No SVG test cases were found!");
}
