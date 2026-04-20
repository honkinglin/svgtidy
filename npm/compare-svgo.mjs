import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import * as bindings from '../pkg/svgtidy_bg.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const defaultTarget = path.resolve(__dirname, '../test-cases');

async function initSvgtidy() {
  const wasmPath = new URL('../pkg/svgtidy_bg.wasm', import.meta.url);
  const wasmBuffer = fs.readFileSync(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBuffer);
  const wasmInstance = await WebAssembly.instantiate(wasmModule, {
    './svgtidy_bg.js': bindings,
  });

  bindings.__wbg_set_wasm(wasmInstance.exports);
}

async function loadSvgoOptimize() {
  try {
    const svgo = await import('svgo');
    return svgo.optimize;
  } catch {
    console.error('Missing dependency: svgo');
    console.error('Install it with: npm --prefix npm install --save-dev svgo');
    process.exit(1);
  }
}

function collectSvgFiles(targetPath) {
  const stat = fs.statSync(targetPath);
  if (stat.isFile()) {
    return targetPath.endsWith('.svg') ? [targetPath] : [];
  }

  const files = [];

  for (const entry of fs.readdirSync(targetPath, { withFileTypes: true })) {
    const entryPath = path.join(targetPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectSvgFiles(entryPath));
    } else if (entry.isFile() && entry.name.endsWith('.svg')) {
      files.push(entryPath);
    }
  }

  return files.sort();
}

function parseArgs(argv) {
  let target = defaultTarget;
  let iterations = 20;

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];

    if (arg === '--iterations' || arg === '-n') {
      const raw = argv[i + 1];
      if (raw == null) {
        throw new Error('Missing value for --iterations');
      }
      iterations = Number.parseInt(raw, 10);
      i += 1;
      continue;
    }

    if (!arg.startsWith('-')) {
      target = path.resolve(process.cwd(), arg);
      continue;
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  if (!Number.isInteger(iterations) || iterations <= 0) {
    throw new Error('--iterations must be a positive integer');
  }

  return { target, iterations };
}

function formatBytes(bytes) {
  return `${bytes} B`;
}

function formatMs(ns, iterations) {
  const avgMs = Number(ns) / iterations / 1e6;
  return avgMs.toFixed(3);
}

function measure(label, fn, iterations) {
  let output = '';
  const started = process.hrtime.bigint();

  for (let i = 0; i < iterations; i += 1) {
    output = fn();
  }

  const elapsed = process.hrtime.bigint() - started;
  return { label, output, elapsed };
}

async function main() {
  const { target, iterations } = parseArgs(process.argv.slice(2));
  const optimizeSvgo = await loadSvgoOptimize();
  await initSvgtidy();

  if (!fs.existsSync(target)) {
    throw new Error(`Target not found: ${target}`);
  }

  const files = collectSvgFiles(target);
  if (files.length === 0) {
    throw new Error(`No SVG files found at: ${target}`);
  }

  const rows = [];
  let totalInputBytes = 0;
  let totalSvgtidyBytes = 0;
  let totalSvgoBytes = 0;
  let totalSvgtidyNs = 0n;
  let totalSvgoNs = 0n;

  for (const file of files) {
    const input = fs.readFileSync(file, 'utf8');
    const relative = path.relative(path.resolve(__dirname, '..'), file);
    const inputBytes = Buffer.byteLength(input, 'utf8');

    const svgtidy = measure(
      'svgtidy',
      () => bindings.optimize(input),
      iterations,
    );
    const svgo = measure(
      'svgo',
      () =>
        optimizeSvgo(input, {
          path: file,
          floatPrecision: 3,
          plugins: ['preset-default'],
        }).data,
      iterations,
    );

    const svgtidyBytes = Buffer.byteLength(svgtidy.output, 'utf8');
    const svgoBytes = Buffer.byteLength(svgo.output, 'utf8');

    totalInputBytes += inputBytes;
    totalSvgtidyBytes += svgtidyBytes;
    totalSvgoBytes += svgoBytes;
    totalSvgtidyNs += svgtidy.elapsed;
    totalSvgoNs += svgo.elapsed;

    rows.push({
      file: relative,
      input: formatBytes(inputBytes),
      svgtidy: formatBytes(svgtidyBytes),
      svgo: formatBytes(svgoBytes),
      size_delta: svgtidyBytes - svgoBytes,
      svgtidy_ms: formatMs(svgtidy.elapsed, iterations),
      svgo_ms: formatMs(svgo.elapsed, iterations),
    });
  }

  console.table(rows);

  console.log('Summary');
  console.log(`Files: ${files.length}`);
  console.log(`Iterations per file: ${iterations}`);
  console.log(`Input total: ${formatBytes(totalInputBytes)}`);
  console.log(`svgtidy total: ${formatBytes(totalSvgtidyBytes)}`);
  console.log(`svgo total: ${formatBytes(totalSvgoBytes)}`);
  console.log(
    `svgtidy avg/file: ${formatMs(totalSvgtidyNs, files.length * iterations)} ms`,
  );
  console.log(
    `svgo avg/file: ${formatMs(totalSvgoNs, files.length * iterations)} ms`,
  );
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
