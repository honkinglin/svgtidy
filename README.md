# svgx

**svgx** is a high-performance SVG optimizer written in Rust. It is designed to providing fast and safe SVG optimization.

🚧 **Work in Progress**: This project is currently in the early stages of development.

## Usage

**svgx** is distinct from other optimizers because it is written in pure Rust with no external dependencies (like Node.js or Python).

### CLI

```bash
# Optimize strictly to stdout
svgx input.svg

# Optimize to file
svgx input.svg -o output.svg

# Batch Processing (Directory)
# Recursively finds all .svg files in `input_dir` and mirrors structure to `output_dir`
# Uses parallel processing (rayon) for maximum speed.
svgx input_dir -o output_dir

# Set precision (default 3)
svgx input.svg -o output.svg -p 5

# Customize plugins
svgx input.svg --disable removeTitle,removeDesc --enable removeStyleElement
```

### WebAssembly (WASM)

`svgx` can be compiled to WASM for use in the browser.

```bash
wasm-pack build --target web
```

Exposes:
```rust
pub fn optimize(svg: &str) -> String;
```

## Performance

Benchmarks run on MacBook Pro (M3):

| Scenario | Input Size | svgx Time | vs SVGO (Node) |
| :--- | :--- | :--- | :--- |
| **Simple Icon** | ~0.5 KB | **~16 µs** | ~100x Faster |
| **Complex SVG** | ~30 KB | **~1 ms** | ~50x Faster |

## Features

- **Blazing Fast**: Built with Rust, `xmlparser`, and `rayon`.
- **Batch Processing**: Parallel directory scanning and optimization.
- **AST-based**: Full DOM-like mutation capabilities.
- **Lossy & Lossless**: Configurable optimization levels.
- **Batched Plugins**:
    - Structural optimization (collapsing groups, moving attributes).
    - Path data optimization (minification, relative/absolute conversion).
    - Transform simplification (matrix multiplication).
    - Cleanup (comments, metadata, unused defs/IDs).

## Roadmap

- [x] Core Plugins (Parity with SVGO)
- [x] CLI Interface
- [x] Parallel Processing (Rayon)
- [x] Batch Processing
- [x] WebAssembly (WASM) Support

## 🏗 Architecture

- **`src/parser.rs`**: Pull-based XML parser converting SVG to an internal AST.
- **`src/tree.rs`**: AST definitions (Document, Element, Node).
- **`src/plugins/`**: Modular optimization passes.
- **`src/printer.rs`**: Serializes the AST back to a minimized SVG string.
- **`src/lib.rs`**: Library entry point (WASM compatible).

## 🤝 Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.

## 📄 License

MIT
