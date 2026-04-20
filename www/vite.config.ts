import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { fileURLToPath, URL } from 'node:url';

// https://vite.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      svgtidy: fileURLToPath(new URL('../pkg/svgtidy.js', import.meta.url)),
    },
  },
  plugins: [
    react(),
    wasm(),
    topLevelAwait()
  ]
})
