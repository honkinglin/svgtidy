# svgx

**svgx** is a high-performance SVG optimizer written in Rust. It is designed to providing fast and safe SVG optimization.

🚧 **Work in Progress**: This project is currently in the early stages of development.

## Usage

**svgx** is distinct from other optimizers because it is written in pure Rust with no external dependencies (like Node.js or Python).

### CLI

```bash
# Optimize strict to stdout
svgx input.svg

# Optimize to file
svgx input.svg -o output.svg

# Set precision (default 3)
svgx input.svg -o output.svg -p 5

# Customize plugins
svgx input.svg --disable removeTitle,removeDesc --enable removeStyleElement
```

## Performance

Benchmarks run on MacBook Pro (M3):

| Scenario | Input Size | svgx Time | vs SVGO (Node) |
| :--- | :--- | :--- | :--- |
| **Simple Icon** | ~0.5 KB | **~16 µs** | ~100x Faster |
| **Complex SVG** | ~30 KB | **~1 ms** | ~50x Faster |

## Features

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
- [ ] Parallel Processing (Rayon)
- [ ] Watch Mode
- [ ] WebAssembly (WASM) Support

## 🚀 Features

- **Blazing Fast**: Built with Rust and `xmlparser` for minimal overhead.
- **AST-based Optimization**: Parses SVG into a DOM-like structure to apply robust transformations.
- **Plugin System**: Modular architecture for optimization passes.
    - `removeComments`: Removes comments from SVG files.
    - *(More plugins coming soon)*
- **CLI**: Simple command-line interface.

## 📦 Installation

Ensure you have [Rust installed](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/yourusername/svgx.git
cd svgx
cargo build --release
```

## 🛠 Usage

Run the `svgx` binary with an input file:

```bash
# Print optimized SVG to stdout
cargo run --release -- input.svg

# Write optimized SVG to a file
cargo run --release -- input.svg -o output.svg
```

### Example

**Input (`test.svg`):**
```xml
<svg width="100" height="100">
    <!-- This is a comment -->
    <rect width="100" height="100" fill="red" />
</svg>
```

**Command:**
```bash
cargo run -- test.svg
```

**Output:**
```xml
<svg width="100" height="100">
    <rect width="100" height="100" fill="red"/>
</svg>
```

## 🏗 Architecture

- **`src/parser.rs`**: Pull-based XML parser converting SVG to an internal AST.
- **`src/tree.rs`**: AST definitions (Document, Element, Node).
- **`src/plugins/`**: collection of optimization plugins implementing the `Plugin` trait.
- **`src/printer.rs`**: Serializes the AST back to a minimized SVG string.

## 🤝 Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.

1. Fork the repo.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes.
4. Push to the branch.
5. Open a Pull Request.

## 📄 License

MIT
