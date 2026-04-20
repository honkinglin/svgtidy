use clap::Parser;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use svgtidy::optimize_with_options;
use svgtidy::pipeline::{unknown_plugin_names, OptimizeOptions};
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

fn build_options(args: &Args) -> Result<OptimizeOptions, String> {
    let options = OptimizeOptions {
        precision: args.precision as usize,
        enable: args.enable.iter().cloned().collect(),
        disable: args.disable.iter().cloned().collect(),
    };

    let unknown = unknown_plugin_names(&options);
    if unknown.is_empty() {
        Ok(options)
    } else {
        Err(format!("Unknown plugin(s): {}", unknown.join(", ")))
    }
}

fn process_string(text: &str, options: &OptimizeOptions) -> Result<String, String> {
    optimize_with_options(text, &options).map_err(|e| format!("Parse error: {}", e))
}

fn main() {
    let args = Args::parse();
    let options = match build_options(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    };

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
                match process_string(&text, &options) {
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
            Ok(text) => match process_string(&text, &options) {
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
