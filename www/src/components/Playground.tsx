import { useState, useMemo, useDeferredValue } from 'react';
import { optimize } from 'svgtidy';
import { Copy, Check, FileCode, Image as ImageIcon } from 'lucide-react';


const DEFAULT_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <!-- This is a comment that will be removed -->
  <rect x="10" y="10" width="80" height="80" fill="red"/>
  <circle cx="50" cy="50" r="20" fill="blue" />
</svg>`;

export function Playground() {
  const [input, setInput] = useState(DEFAULT_SVG);
  const deferredInput = useDeferredValue(input);
  const [copied, setCopied] = useState(false);
  const [viewMode, setViewMode] = useState<'preview' | 'code'>('preview');

  const { output, error } = useMemo(() => {
    if (!deferredInput.trim()) {
      return { output: '', error: null };
    }
    try {
      const result = optimize(deferredInput);
      return { output: result, error: null };
    } catch (err) {
      console.error(err);
      return { output: '', error: "Failed to optimize SVG. Ensure input is valid XML." };
    }
  }, [deferredInput]);

  const handleCopy = () => {
    navigator.clipboard.writeText(output);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const stats = {
    original: new Blob([deferredInput]).size,
    optimized: new Blob([output]).size,
  };
  const savings = stats.original > 0 
    ? ((stats.original - stats.optimized) / stats.original * 100).toFixed(1) 
    : 0;

  return (
    <section id="playground" className="py-20 bg-surface border-t border-border">
      <div className="container mx-auto px-5">
        <div className="text-center mb-10">
           <h2 className="text-3xl font-bold mb-3">Interactive Playground</h2>
           <p className="text-text-muted">Paste your SVG below and see the WASM optimizer in action.</p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 h-auto min-h-[600px]">
          {/* Input Panel */}
          <div className="flex flex-col bg-bg border border-border rounded-xl overflow-hidden shadow-sm">
            <div className="flex justify-between items-center px-4 py-3 bg-surface border-b border-border h-[50px]">
              <span className="font-semibold text-sm">Input SVG</span>
              <span className="text-xs px-2 py-1 bg-border rounded text-text-muted font-mono">{stats.original} bytes</span>
            </div>
            <textarea 
              className="flex-1 w-full p-4 font-mono text-sm leading-relaxed bg-transparent border-none resize-none outline-none focus:ring-0 text-text" 
              value={input} 
              onChange={(e) => setInput(e.target.value)}
              placeholder="Paste SVG code here..."
              spellCheck={false}
            />
          </div>

          {/* Output Panel */}
          <div className="flex flex-col bg-bg border border-border rounded-xl overflow-hidden shadow-sm">
            <div className="flex justify-between items-center px-4 py-3 bg-surface border-b border-border h-[50px]">
              <span className="font-semibold text-sm">Optimized SVG</span>
              <div className="flex items-center gap-3">
                 <span className="text-xs px-2 py-1 rounded font-mono bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400">
                   {stats.optimized} bytes (-{savings}%)
                 </span>
                 <div className="flex bg-border rounded p-0.5 gap-0.5">
                    <button 
                        className={`p-1.5 rounded flex items-center justify-center transition-colors ${viewMode === 'preview' ? 'bg-surface text-primary shadow-sm' : 'text-text-muted hover:text-text hover:bg-black/5 dark:hover:bg-white/5'}`}
                        onClick={() => setViewMode('preview')}
                        title="Preview"
                    >
                        <ImageIcon size={16} />
                    </button>
                    <button 
                        className={`p-1.5 rounded flex items-center justify-center transition-colors ${viewMode === 'code' ? 'bg-surface text-primary shadow-sm' : 'text-text-muted hover:text-text hover:bg-black/5 dark:hover:bg-white/5'}`}
                        onClick={() => setViewMode('code')}
                        title="View Code"
                    >
                        <FileCode size={16} />
                    </button>
                 </div>
                 <button 
                    className="p-1.5 rounded flex items-center justify-center text-text-muted hover:text-text hover:bg-black/5 dark:hover:bg-white/5 transition-colors" 
                    onClick={handleCopy} 
                    title="Copy to Clipboard"
                 >
                   {copied ? <Check size={16} /> : <Copy size={16} />}
                 </button>
              </div>
            </div>
            
            <div className="flex-1 flex flex-col overflow-hidden relative">
                {error ? (
                    <div className="p-5 text-red-500 text-sm">{error}</div>
                ) : (
                    viewMode === 'preview' ? (
                        <div 
                            className="flex-1 flex items-center justify-center bg-[radial-gradient(#e5e7eb_1px,transparent_1px)] dark:bg-[radial-gradient(#334155_1px,transparent_1px)] [background-size:20px_20px] p-6 min-h-[300px]"
                            dangerouslySetInnerHTML={{ __html: output }} 
                        />
                    ) : (
                        <textarea 
                            className="flex-1 w-full p-4 font-mono text-sm leading-relaxed bg-black/5 dark:bg-white/5 border-none resize-none outline-none text-text" 
                            value={output} 
                            readOnly 
                        />
                    )
                )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
