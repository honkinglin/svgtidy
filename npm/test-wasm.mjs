import * as bindings from './svgtidy-wasm/svgtidy_bg.js';
import fs from 'fs';

async function testWasm() {
    console.log("🧪 Testing WASM Core (Manual Node.js Instantiation)...");

    // Read the WASM file
    const wasmPath = new URL('./svgtidy-wasm/svgtidy_bg.wasm', import.meta.url);
    const wasmBuffer = fs.readFileSync(wasmPath);

    // Instantiate directly
    const wasmModule = await WebAssembly.compile(wasmBuffer);
    const wasmInstance = await WebAssembly.instantiate(wasmModule, {
        './svgtidy_bg.js': bindings 
    });

    // Link bindings to WASM instance
    bindings.__wbg_set_wasm(wasmInstance.exports);

    // Run Test
    const input = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect x="10" y="10" width="80" height="80" fill="red"/></svg>';
    try {
        const output = bindings.optimize(input);
        
        if (output.includes('<path') && !output.includes('<rect')) {
            console.log("✅ WASM Test Passed: Rect converted to path.");
            console.log("   Input: " + input);
            console.log("   Output: " + output);
        } else {
            console.error("❌ WASM Test Failed: Output unexpected.");
            console.log("   Output: " + output);
            process.exit(1);
        }
    } catch (e) {
        console.error("❌ Execution Error:", e);
        process.exit(1);
    }
}

testWasm();
