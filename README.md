# svgx

![CI](https://github.com/honkinglin/svgx/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

> **svgx** is a high-performance SVG optimizer written in Rust.

It removes redundant information from SVG files (like comments, metadata, and hidden elements) and creates a minimized, cleaner version without affecting rendering.

Compared to [SVGO](https://github.com/svg/svgo), `svgx` is **fast**—up to 100x faster for single icons and 50x faster for complex files.

---

## ⚡ Features

- **Blazing Fast**: Built with Rust, `xmlparser` and `rayon`.
- **Batch Processing**: Parallel directory scanning and optimization.
- **AST-based**: Robust DOM-like mutations.
- **Configurable**: Toggle plugins, set precision, and precision formatting.
- **WASM Support**: Running directly in the browser (coming soon to a web UI).

## 🚀 Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

### From Source
```bash
git clone https://github.com/honkinglin/svgx.git
cd svgx
cargo install --path .
```
This will compile the project and install the `svgx` binary to your Cargo bin directory (usually `~/.cargo/bin`). Ensure this directory is in your `PATH`.

Alternatively, to build without installing:
```bash
cargo build --release
# Binary will be at ./target/release/svgx
```

## 🛠 Usage

### Command Line (CLI)

**Basic Optimization**
```bash
svgx input.svg -o output.svg
```

**Directory (Batch) Mode**
Recursively optimizes all SVGs in `icons/` and saves them to `dist/`, maintaining directory structure.
```bash
svgx icons/ -o dist/
```

**Customization**
```bash
# Set numeric precision to 5 decimal places
svgx input.svg -o output.svg -p 5

# Enable/Disable specific plugins
svgx input.svg --disable removeTitle --enable removeStyleElement
```

### Full CLI Options
```text
Usage: svgx [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file or directory

Options:
  -o, --output <OUTPUT>    Output file or directory
  -p, --precision <PRECISION>  Set numeric precision [default: 3]
      --enable <ENABLE>    Enable specific plugins (comma-separated)
      --disable <DISABLE>  Disable specific plugins (comma-separated)
      --pretty             Pretty print output
  -h, --help               Print help
```

## 🔌 Plugins

`svgx` currently supports the following optimization plugins:

| Plugin Name | Description | Default |
| :--- | :--- | :--- |
| `removeDoctype` | Removes `<!DOCTYPE>` declaration. | `true` |
| `removeXMLProcInst` | Removes `<?xml ... ?>` instructions. | `true` |
| `removeComments` | Removes comments. | `true` |
| `removeMetadata` | Removes `<metadata>` elements. | `true` |
| `removeTitle` | Removes `<title>` elements. | `true` |
| `removeDesc` | Removes `<desc>` elements. | `true` |
| `removeEditorsNSData`| Removes editor namespaced attributes (Inkscape, etc.). | `true` |
| `cleanupAttrs` | Trims attribute whitespace. | `true` |
| `mergePaths` | Merges adjacent paths with same attributes. | `true` |
| `convertShapeToPath` | Converts basic shapes (rect, circle) to path. | `true` |
| `convertPathData` | Optimizes path commands (relative, precision). | `true` |
| `convertTransform` | Collapses multiple transforms into one. | `true` |
| `removeHiddenElems` | Removes hidden elements (`display="none"`). | `true` |
| `removeEmptyText` | Removes empty text nodes. | `true` |
| `convertColors` | Converts colors (rgb to hex, etc.). | `true` |
| `collapseGroups` | Removes redundant `<g>` tags. | `true` |
| `moveGroupAttrsToElems`| Moves attributes from groups to elements (enabling collapse). | `true` |
| `moveElemsAttrsToGroup`| Moves common attributes from elements to groups. | `true` |

*(And many more...)*

## 📊 Benchmarks

Tests performed on a MacBook Pro (M3).

| Scenario | Input Size | svgx Time | vs SVGO (Node) |
| :--- | :--- | :--- | :--- |
| **Simple Icon** | ~0.5 KB | **~16 µs** | **~100x Faster** |
| **Complex SVG** | ~30 KB | **~1 ms** | **~50x Faster** |

## 🕸 WebAssembly (WASM)

`svgx` provides a WASM interface for web usage.

```bash
wasm-pack build --target web
```

**JS Example:**
```javascript
import init, { optimize } from './pkg/svgx.js';

await init();
const output = optimize('<svg>...</svg>');
console.log(output);
```

## 🏗 Architecture

The optimization pipeline is purely AST-based:
1.  **Parser**: `xmlparser` (pull-parser) -> `Document` (DOM Tree).
2.  **Plugins**: Vists and mutates the `Document` tree.
3.  **Printer**: Serializes `Document` back to string.

This allows for complex, context-aware optimizations (like moving attributes up/down the tree) that regex-based tools cannot safely perform.

## 🤝 Contributing

Contributions are welcome!

1.  Fork the repository.
2.  Create a feature branch.
3.  Submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
