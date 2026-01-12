# ![Logo](logo.svg)

![CI](https://github.com/honkinglin/svgx/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

> **svgtidy** is a high-performance SVG optimizer written in Rust.

It removes redundant information from SVG files (like comments, metadata, and hidden elements) and creates a minimized, cleaner version without affecting rendering.

Compared to [SVGO](https://github.com/svg/svgo), `svgtidy` is **fast**—up to 100x faster for single icons and 50x faster for complex files.

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
cd svgtidy
cargo install --path .
```
This will compile the project and install the `svgtidy` binary to your Cargo bin directory (usually `~/.cargo/bin`). Ensure this directory is in your `PATH`.

Alternatively, to build without installing:
```bash
cargo build --release
# Binary will be at ./target/release/svgtidy
```

## 🛠 Usage

### Command Line (CLI)

**Basic Optimization**
```bash
svgtidy input.svg -o output.svg
```

**Directory (Batch) Mode**
Recursively optimizes all SVGs in `icons/` and saves them to `dist/`, maintaining directory structure.
```bash
svgtidy icons/ -o dist/
```

**Customization**
```bash
# Set numeric precision to 5 decimal places
svgtidy input.svg -o output.svg -p 5

# Enable/Disable specific plugins
svgtidy input.svg --disable removeTitle --enable removeStyleElement
```

### Full CLI Options
```text
Usage: svgtidy [OPTIONS] <INPUT>

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

`svgtidy` currently supports the following optimization plugins:

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

| Scenario | Input Size | svgtidy Time | vs SVGO (Node) |
| :--- | :--- | :--- | :--- |
| **Simple Icon** | ~0.5 KB | **~16 µs** | **~100x Faster** |
| **Complex SVG** | ~30 KB | **~1 ms** | **~50x Faster** |

## 🕸 WebAssembly (WASM)

`svgtidy` provides a WASM interface for web usage.

```bash
wasm-pack build --target web
```

**JS Example:**
```javascript
import init, { optimize } from './pkg/svgtidy.js';

await init();
const output = optimize('<svg>...</svg>');
console.log(output);
```

## 📦 Javascript Ecosystem

`svgtidy` is available for the Javascript ecosystem via NPM.

### Vite Plugin
```bash
npm install vite-plugin-svgtidy
```
```javascript
// vite.config.js
import svgtidy from 'vite-plugin-svgtidy';

export default {
  plugins: [svgtidy()]
}
```

### Webpack Loader
```bash
npm install svgtidy-loader
```
```javascript
// webpack.config.js
module.exports = {
  module: {
    rules: [
      {
        test: /\.svg$/,
        use: [
          { loader: 'svgtidy-loader' }
        ]
      }
    ]
  }
}
```

## 🛠 Development of Plugins

The JS packages are located in the `npm/` directory.

### Build WASM Core
First, build the core WASM module that plugins depend on:
```bash
wasm-pack build --target bundler --out-dir npm/svgtidy-wasm
```

### Build & Test Plugins
To verify the plugins work correctly with the built WASM:
```bash
cd npm
node verify.mjs
```
This script will:
1. Verify the bare WASM module.
2. Build `vite-plugin-svgtidy` and `svgtidy-loader`.
3. Run example projects (`npm/examples/`) to verify real-world usage.



## 🧪 Testing

`svgtidy` uses a unified test suite to ensure consistency between the Rust core and the WASM/JS implementations.

### Shared Test Cases
Test cases are located in the `test-cases/` directory at the project root. To add a new test case, simply add an SVG file to this directory.

### Running Tests

**Rust Core**:
```bash
cargo test
```
This runs unit tests and the integration test `tests/integration_test.rs`, which verifies all SVGs in `test-cases/`.

**JS/WASM**:
```bash
cd npm
npm run test:suite
```
This runs the Node.js test script `npm/test-suite.mjs`, which also verifies all SVGs in `test-cases/` using the compiled WASM module.

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
