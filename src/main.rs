use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use svgx::parser;
use svgx::plugins::{
    CleanupAttrs, CleanupIds, CleanupListOfValues, CleanupNumericValues, CollapseGroups,
    ConvertColors, ConvertEllipseToCircle, ConvertOneStopGradients, ConvertPathData,
    ConvertShapeToPath, ConvertStyleToAttrs, ConvertTransform, MergePaths, MoveElemsAttrsToGroup,
    MoveGroupAttrsToElems, Plugin, RemoveComments, RemoveDesc, RemoveDimensions, RemoveDoctype,
    RemoveEditorsNSData, RemoveEmptyAttrs, RemoveEmptyContainers, RemoveEmptyText,
    RemoveHiddenElems, RemoveMetadata, RemoveRasterImages, RemoveScriptElement, RemoveStyleElement,
    RemoveTitle, RemoveUnknownsAndDefaults, RemoveUnusedNS, RemoveUselessDefs,
    RemoveUselessStrokeAndFill, RemoveXMLProcInst, SortAttrs, SortDefsChildren,
};
use svgx::printer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input SVG file
    input: PathBuf,

    /// Output SVG file (optional, prints to stdout if not provided)
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

fn main() {
    let args = Args::parse();

    // Read input
    let text = fs::read_to_string(&args.input).expect("Could not read input file");

    // Parse
    match parser::parse(&text) {
        Ok(mut doc) => {
            // Configure Plugins
            let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

            // Helper to create config
            let p =
                |name: &'static str, factory: Box<dyn Fn() -> Box<dyn Plugin>>, default: bool| {
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
            // args.enable overrides default false
            // args.disable overrides default true

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

            // Apply Plugins
            for plugin in plugins {
                plugin.apply(&mut doc);
            }

            // Output
            let out = printer::print(&doc);

            if let Some(output_path) = args.output {
                fs::write(output_path, out).expect("Could not write output file");
            } else {
                println!("{}", out);
            }
        }
        Err(e) => {
            eprintln!("Error parsing SVG: {}", e);
            std::process::exit(1);
        }
    }
}
