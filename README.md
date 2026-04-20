# ![Logo](logo.svg)

![CI](https://github.com/honkinglin/svgtidy/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Crates.io Version](https://img.shields.io/crates/v/svgtidy)
![NPM Version](https://img.shields.io/npm/v/svgtidy)

> **svgtidy** is a high-performance SVG optimizer written in Rust.

It removes redundant information from SVG files (like comments, metadata, and hidden elements) and creates a minimized, cleaner version without affecting rendering.

Compared to [SVGO](https://github.com/svg/svgo), `svgtidy` is designed to be fast while preserving SVG semantics.

---

## ⚡ Features

- **Blazing Fast**: Built with Rust, `xmlparser` and `rayon`.
- **Batch Processing**: Parallel directory scanning and optimization.
- **AST-based**: Robust DOM-like mutations.
- **Configurable**: Toggle plugins, set precision, and precision formatting.
- **Cross-Platform**: Runs on CLI, Node.js, and in the browser (WASM).

## 🚀 Installation & Usage

### 📦 Node.js / Web (WASM)

Use `svgtidy` directly in your JavaScript/TypeScript projects.

```bash
npm install svgtidy
```

**Usage:**

```javascript
import { optimize } from 'svgtidy';

const svg = '<svg>...</svg>';
const optimized = optimize(svg);
console.log(optimized);
```

### 🦀 CLI (Command Line)

Install the binary tool using Rust's cargo:

```bash
# Install from crates.io (Recommended)
cargo install svgtidy

# Or build from source
git clone https://github.com/honkinglin/svgtidy.git
cd svgtidy
cargo install --path .
```

**Usage:**

```bash
# Optimize a single file
svgtidy input.svg -o output.svg

# Optimize a directory (recursive)
svgtidy icons/ -o dist/

# Set precision and disable specific plugins
svgtidy input.svg -o output.svg -p 5 --disable removeTitle
```

### ⚡ Vite

Install the dedicated Vite plugin:

```bash
npm install vite-plugin-svgtidy
```

**Usage (`vite.config.js`):**

```javascript
import svgtidy from 'vite-plugin-svgtidy';

export default {
  plugins: [svgtidy()]
}
```

### 📦 Webpack

Install the Webpack loader:

```bash
npm install svgtidy-loader
```

**Usage (`webpack.config.js`):**

```javascript
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

## ⚙️ Configuration (CLI)

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

`svgtidy` enables these plugins by default to ensure maximum reduction:

| Plugin Name | Description |
| :--- | :--- |
| `removeDoctype` | Removes `<!DOCTYPE>` declaration. |
| `removeXMLProcInst` | Removes `<?xml ... ?>` instructions. |
| `removeComments` | Removes comments. |
| `removeMetadata` | Removes `<metadata>` elements. |
| `removeTitle` | Removes `<title>` elements. |
| `removeDesc` | Removes `<desc>` elements. |
| `removeEditorsNSData`| Removes editor namespaced attributes (Inkscape, etc.). |
| `cleanupAttrs` | Trims attribute whitespace. |
| `mergePaths` | Conservatively merges adjacent simple paths when explicitly enabled. |
| `convertShapeToPath` | Converts selected basic shapes (rect, line, poly*) to path. |
| `convertPathData` | Optimizes path commands (relative, precision). |
| `convertTransform` | Collapses multiple transforms into one. |
| `removeNonInheritableGroupAttrs` | Removes non-inheritable presentation attributes from `<g>`. |
| `removeHiddenElems` | Removes hidden elements (`display="none"`). |
| `removeEmptyText` | Removes empty text nodes. |
| `convertColors` | Converts colors (rgb to hex, etc.). |
| `collapseGroups` | Removes redundant `<g>` tags. |
| `moveGroupAttrsToElems`| Moves attributes from groups to elements. |
| `moveElemsAttrsToGroup`| Moves common attributes from elements to groups. |

*(And more...)*

## 📊 Benchmarks

| Scenario | Input Size | svgtidy Time | vs SVGO (Node) |
| :--- | :--- | :--- | :--- |
| **Simple Icon** | ~0.5 KB | **~16 µs** | **~100x Faster** |
| **Complex SVG** | ~30 KB | **~1 ms** | **~50x Faster** |

## � Development

### Build WASM locally

To build the WASM package for web usage (NPM):

```bash
wasm-pack build --target bundler --out-dir npm/svgtidy-wasm
```

### Running Tests

- **Rust**: `cargo test`
- **JS/WASM**: `cd npm && npm run test:suite`

### Refreshing pkg

Regenerate the checked-in browser bundle with:

```bash
./scripts/build-pkg.sh
```

The script removes the `.gitignore` recreated by `wasm-pack` and cleans legacy `svgx*` files so `pkg/` stays trackable and deterministic in git.

### Comparing Against SVGO

Install `svgo` in the npm workspace:

```bash
npm --prefix npm install --save-dev svgo
```

Then compare `svgtidy` against SVGO on the default `test-cases/` corpus:

```bash
npm --prefix npm run compare:svgo
```

You can also point it at a specific file or directory and control the number of timing iterations:

```bash
npm --prefix npm run compare:svgo -- ../test-cases --iterations 50
```

## 🤝 Contributing

Contributions are welcome!

1.  Fork the repository.
2.  Create a feature branch.
3.  Submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
