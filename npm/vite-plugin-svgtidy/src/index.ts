import type { Plugin } from 'vite';
import { optimize, initWasm } from './wasm-load';
import fs from 'node:fs';

export default function svgtidyPlugin(options: any = {}): Plugin {
  return {
    name: 'vite-plugin-svgtidy',
    enforce: 'pre',
    async buildStart() {
        await initWasm();
    },
    async transform(code: string, id: string) {
      if (id.endsWith('.svg')) {
        // Only optimize if it's imported as raw SVG or handled by another loader?
        // Typically Vite handles SVGs as URLs. 
        // If query param ?raw is present, Vite returns string.
        
        // Let's optimize content.
        // We need the raw content. 'code' might be the URL export from Vite default.
        // So we read the file.
        const cleanId = id.split('?')[0];
        const content = fs.readFileSync(cleanId, 'utf-8');
        
        try {
            const optimized = optimize(content);
            return {
                code: `export default ${JSON.stringify(optimized)}`,
                map: null
            }
        } catch (e) {
            console.error('svgtidy error:', e);
        }
      }
      return null;
    }
  };
}
