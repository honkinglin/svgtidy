import fs from 'node:fs';
import { createRequire } from 'node:module';
// @ts-ignore - Internal bindings do not have types
import * as bindings from 'svgtidy-wasm/svgtidy_bg.js';

let wasmLoaded = false;

export async function initWasm() {
    if (wasmLoaded) return;

    const require = createRequire(import.meta.url);
    
    // Resolve path to .wasm file
    // We assume svgtidy-wasm is installed and accessible
    const wasmPath = require.resolve('svgtidy-wasm/svgtidy_bg.wasm');
    
    const wasmBuffer = fs.readFileSync(wasmPath);
    
    const wasmModule = new WebAssembly.Module(wasmBuffer);
    const wasmInstance = new WebAssembly.Instance(wasmModule, {
        './svgtidy_bg.js': bindings
    });

    // Initialize bindings with the WASM instance exports
    // @ts-ignore - __wbg_set_wasm is internal but exposed in bg.js
    bindings.__wbg_set_wasm(wasmInstance.exports);
    
    // If there's a start function, call it (some wasm-pack targets have it)
    // @ts-ignore
    if (wasmInstance.exports.__wbindgen_start) {
        // @ts-ignore
        wasmInstance.exports.__wbindgen_start();
    }

    wasmLoaded = true;
}

export function optimize(svg: string): string {
    if (!wasmLoaded) {
        throw new Error("WASM not initialized. Call initWasm() first.");
    }
    return bindings.optimize(svg);
}
