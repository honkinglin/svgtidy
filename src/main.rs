use clap::Parser;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
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
use walkdir::WalkDir;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file or directory
    input: PathBuf,

    /// Output file or directory (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Set numeric precision
    #[arg(short, long, default_value_t = 3)]
    precision: u8,

    /// Enable specific plugins (comma-separated list)
    #[arg(long, value_delimiter = ',')]
    enable: Vec<String>,

    /// Disable specific plugins (comma-separated list)
    #[arg(long, value_delimiter = ',')]
    disable: Vec<String>,

    /// Pretty print output (disable minification)
    #[arg(long)]
    pretty: bool,
}

struct PluginConfig {
    name: &'static str,
    factory: Box<dyn Fn() -> Box<dyn Plugin>>,
    enabled_by_default: bool,
}

impl PluginConfig {
    fn new(
        name: &'static str,
        factory: Box<dyn Fn() -> Box<dyn Plugin>>,
        enabled_by_default: bool,
    ) -> Self {
        Self {
            name,
            factory,
            enabled_by_default,
        }
    }
}

fn get_config(args: &Args) -> Vec<Box<dyn Plugin>> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

    // Helper to create config
    let p = |name: &'static str, factory: Box<dyn Fn() -> Box<dyn Plugin>>, default: bool| {
        PluginConfig::new(name, factory, default)
    };

    // Capture precision for closures (cast to usize)
    let precision = args.precision as usize;

    // Define all available plugins
    // Note: Order matters for optimal processing!
    let available_plugins = vec![
        p("removeDoctype", Box::new(|| Box::new(RemoveDoctype)), true),
        p(
            "removeXMLProcInst",
            Box::new(|| Box::new(RemoveXMLProcInst)),
            true,
        ),
        p(
            "removeComments",
            Box::new(|| Box::new(RemoveComments)),
            true,
        ),
        p(
            "removeMetadata",
            Box::new(|| Box::new(RemoveMetadata)),
            true,
        ),
        p("removeTitle", Box::new(|| Box::new(RemoveTitle)), true),
        p("removeDesc", Box::new(|| Box::new(RemoveDesc)), true),
        p(
            "removeEditorsNSData",
            Box::new(|| Box::new(RemoveEditorsNSData)),
            true,
        ),
        p(
            "removeScriptElement",
            Box::new(|| Box::new(RemoveScriptElement)),
            true,
        ),
        p(
            "removeRasterImages",
            Box::new(|| Box::new(RemoveRasterImages)),
            true,
        ),
        p(
            "removeStyleElement",
            Box::new(|| Box::new(RemoveStyleElement)),
            false,
        ), // Optional
        p(
            "convertStyleToAttrs",
            Box::new(|| Box::new(ConvertStyleToAttrs)),
            true,
        ),
        p("cleanupAttrs", Box::new(|| Box::new(CleanupAttrs)), true),
        p(
            "removeUselessStrokeAndFill",
            Box::new(|| Box::new(RemoveUselessStrokeAndFill)),
            true,
        ),
        p(
            "removeDimensions",
            Box::new(|| Box::new(RemoveDimensions)),
            true,
        ),
        // Structure
        p(
            "moveGroupAttrsToElems",
            Box::new(|| Box::new(MoveGroupAttrsToElems)),
            true,
        ),
        p(
            "moveElemsAttrsToGroup",
            Box::new(|| Box::new(MoveElemsAttrsToGroup)),
            true,
        ),
        p(
            "convertOneStopGradients",
            Box::new(|| Box::new(ConvertOneStopGradients)),
            true,
        ),
        p("cleanupIds", Box::new(|| Box::new(CleanupIds)), true),
        p(
            "removeUselessDefs",
            Box::new(|| Box::new(RemoveUselessDefs)),
            true,
        ),
        p(
            "removeEmptyContainers",
            Box::new(|| Box::new(RemoveEmptyContainers)),
            true,
        ),
        p(
            "removeHiddenElems",
            Box::new(|| Box::new(RemoveHiddenElems)),
            true,
        ),
        p(
            "removeEmptyText",
            Box::new(|| Box::new(RemoveEmptyText)),
            true,
        ),
        p(
            "collapseGroups",
            Box::new(|| Box::new(CollapseGroups)),
            true,
        ),
        // Shapes & Paths
        p(
            "convertEllipseToCircle",
            Box::new(|| Box::new(ConvertEllipseToCircle)),
            true,
        ),
        p(
            "convertShapeToPath",
            Box::new(|| Box::new(ConvertShapeToPath)),
            true,
        ),
        // Configurable Plugins
        p(
            "convertPathData",
            Box::new(move || {
                Box::new(ConvertPathData {
                    float_precision: precision,
                    leading_zero: true,
                    ..Default::default()
                })
            }),
            true,
        ),
        p(
            "convertTransform",
            Box::new(move || {
                Box::new(ConvertTransform {
                    float_precision: precision,
                    deg_precision: precision,
                    ..Default::default()
                })
            }),
            true,
        ),
        p(
            "cleanupNumericValues",
            Box::new(move || {
                Box::new(CleanupNumericValues {
                    float_precision: precision,
                    remove_px: true,
                    leading_zero: true,
                })
            }),
            true,
        ),
        p(
            "cleanupListOfValues",
            Box::new(move || {
                Box::new(CleanupListOfValues {
                    float_precision: precision,
                    default_px: true,
                    convert_to_px: true,
                    leading_zero: true,
                })
            }),
            true,
        ),
        p(
            "removeUnknownsAndDefaults",
            Box::new(|| Box::new(RemoveUnknownsAndDefaults::default())),
            true,
        ),
        p("mergePaths", Box::new(|| Box::new(MergePaths)), true),
        p("convertColors", Box::new(|| Box::new(ConvertColors)), true),
        p(
            "removeEmptyAttrs",
            Box::new(|| Box::new(RemoveEmptyAttrs)),
            true,
        ),
        p(
            "removeUnusedNS",
            Box::new(|| Box::new(RemoveUnusedNS)),
            true,
        ),
        p("sortAttrs", Box::new(|| Box::new(SortAttrs)), true),
        p(
            "sortDefsChildren",
            Box::new(|| Box::new(SortDefsChildren)),
            true,
        ),
    ];

    // Resolve enabled/disabled plugins
    let explicit_enable: HashSet<&String> = args.enable.iter().collect();
    let explicit_disable: HashSet<&String> = args.disable.iter().collect();

    for config in available_plugins {
        let mut active = config.enabled_by_default;

        if explicit_enable.contains(&config.name.to_string()) {
            active = true;
        }
        if explicit_disable.contains(&config.name.to_string()) {
            active = false;
        }

        if active {
            plugins.push((config.factory)());
        }
    }

    plugins
}

fn process_string(text: &str, args: &Args) -> Result<String, String> {
    match parser::parse(text) {
        Ok(mut doc) => {
            let plugins = get_config(args);
            for plugin in plugins {
                plugin.apply(&mut doc);
            }
            Ok(printer::print(&doc))
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

fn main() {
    let args = Args::parse();

    if args.input.is_dir() {
        // Batch Mode
        let walker = WalkDir::new(&args.input).into_iter();

        // Collect files first to parallelize
        let files: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "svg"))
            .map(|e| e.path().to_owned())
            .collect();

        println!("Found {} SVG files. Processing in parallel...", files.len());

        files.par_iter().for_each(|input_path| {
            // Calculate output path
            let output_path = if let Some(ref out_dir) = args.output {
                // Mirror structure: out_dir + (input_path - args.input)
                let relative = input_path.strip_prefix(&args.input).unwrap();
                Some(out_dir.join(relative))
            } else {
                None // If no output dir, maybe print? Or overwrite? Let's safeguard and strictly require output dir for batch OR just print (too noisy).
                     // Safest: require output dir for batch for now, or just don't write.
            };

            if let Ok(text) = fs::read_to_string(input_path) {
                match process_string(&text, &args) {
                    Ok(out) => {
                        if let Some(path) = output_path {
                            // Ensure parent exists
                            if let Some(parent) = path.parent() {
                                let _ = fs::create_dir_all(parent);
                            }
                            if let Err(e) = fs::write(&path, out) {
                                eprintln!("Error writing {:?}: {}", path, e);
                            } else {
                                // Success silent?
                            }
                        } else {
                            // If no output, just print summary? Or dry run?
                        }
                    }
                    Err(e) => eprintln!("Error processing {:?}: {}", input_path, e),
                }
            }
        });

        println!("Done.");
    } else {
        // Single File Mode
        match fs::read_to_string(&args.input) {
            Ok(text) => match process_string(&text, &args) {
                Ok(out) => {
                    if let Some(output_path) = args.output {
                        fs::write(output_path, out).expect("Could not write output file");
                    } else {
                        println!("{}", out);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Could not read input file: {}", e);
                std::process::exit(1);
            }
        }
    }
}
