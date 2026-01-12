# Documentation

**svgtidy** is a high-performance SVG optimizer written in Rust. It removes redundant information from SVG files (like comments, metadata, and hidden elements) and creates a minimized, cleaner version without affecting rendering.

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

### WebAssembly (WASM)
Running directly in the browser or Node.js.

```javascript
import init, { optimize } from 'svgtidy-wasm';

await init();
const output = optimize('<svg>...</svg>');
console.log(output);
```

## 🔌 Configuration & Plugins

`svgtidy` enables these optimizations by default:

| Plugin Name | Description |
| :--- | :--- |
| `removeDoctype` | Removes `<!DOCTYPE>` declaration. |
| `removeXMLProcInst` | Removes `<?xml ... ?>` instructions. |
| `removeComments` | Removes comments. |
| `removeMetadata` | Removes `<metadata>` elements. |
| `removeTitle` | Removes `<title>` elements. |
| `removeDesc` | Removes `<desc>` elements. |
| `removeHiddenElems` | Removes hidden elements (`display="none"`). |
| `convertShapeToPath` | Converts basic shapes (rect, circle) to path. |
| `convertPathData` | Optimizes path commands (relative, precision). |
| `convertTransform` | Collapses multiple transforms into one. |
| `convertColors` | Converts colors (rgb to hex, etc.). |
| `collapseGroups` | Removes redundant `<g>` tags. |
