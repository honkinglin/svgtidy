import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import * as bindings from './svgtidy-wasm/svgtidy_bg.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testCasesDir = path.join(__dirname, '../test-cases');

async function initWasm() {
    console.log("🔄 Initializing WASM...");
    const wasmPath = new URL('./svgtidy-wasm/svgtidy_bg.wasm', import.meta.url);
    const wasmBuffer = fs.readFileSync(wasmPath);
    const wasmModule = await WebAssembly.compile(wasmBuffer);
    const wasmInstance = await WebAssembly.instantiate(wasmModule, {
        './svgtidy_bg.js': bindings
    });
    bindings.__wbg_set_wasm(wasmInstance.exports);
    console.log("✅ WASM Initialized.");
}

async function runTests() {
    await initWasm();

    if (!fs.existsSync(testCasesDir)) {
        console.error(`❌ Test cases directory not found: ${testCasesDir}`);
        process.exit(1);
    }

    const files = fs.readdirSync(testCasesDir).filter(f => f.endsWith('.svg'));
    if (files.length === 0) {
        console.warn("⚠️ No SVG files found in test-cases directory.");
        return;
    }

    console.log(`\n📂 Found ${files.length} test cases in ${testCasesDir}\n`);

    let passed = 0;
    let failed = 0;

    for (const file of files) {
        const filePath = path.join(testCasesDir, file);
        const input = fs.readFileSync(filePath, 'utf-8');
        const inputSize = Buffer.byteLength(input, 'utf8');

        console.log(`🧪 Testing: ${file} (${inputSize} bytes)`);

        try {
            const output = bindings.optimize(input);
            const outputSize = Buffer.byteLength(output, 'utf8');
            const savings = ((inputSize - outputSize) / inputSize * 100).toFixed(2);

            if (!output.trim().startsWith('<svg')) {
                 throw new Error("Output does not start with <svg");
            }

            console.log(`   ✅ Optimized: ${outputSize} bytes (${savings}% savings)`);
            passed++;
        } catch (e) {
            console.error(`   ❌ Failed: ${e.message}`);
            failed++;
        }
    }

    console.log(`\n📊 Summary: ${passed} passed, ${failed} failed.`);
    if (failed > 0) process.exit(1);
}

runTests().catch(e => {
    console.error("❌ Fatal Error:", e);
    process.exit(1);
});
