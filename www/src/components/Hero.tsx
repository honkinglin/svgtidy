import { Link } from 'react-router-dom';
import { ArrowRight, Zap, Globe, Cpu } from 'lucide-react';


export function Hero() {
  const scrollToPlayground = () => {
    document.getElementById('playground')?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <section className="py-20 text-center">
      <div className="container mx-auto px-5 flex flex-col items-center gap-16">
        <div className="max-w-4xl">
          <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight mb-6 leading-tight text-text">
            Optimize your SVGs <br/>
            <span className="text-transparent bg-clip-text bg-gradient-to-br from-primary to-orange-500">lightning fast.</span>
          </h1>
          <p className="text-xl md:text-2xl text-text-muted max-w-2xl mx-auto mb-10 leading-relaxed">
            A high-performance SVG optimizer written in Rust. 
            Available as a CLI, generic library, and WebAssembly module.
          </p>
          <div className="flex flex-wrap gap-4 justify-center">
            <button 
                className="inline-flex items-center gap-2 px-6 py-3 bg-primary text-white rounded-lg font-semibold hover:bg-primary-dark transition-colors shadow-lg hover:shadow-xl hover:-translate-y-0.5" 
                onClick={scrollToPlayground}
            >
              Try Online <ArrowRight size={20} />
            </button>
            <Link 
                to="/docs" 
                className="inline-flex items-center gap-2 px-6 py-3 bg-transparent border border-border text-text rounded-lg font-semibold hover:bg-surface hover:border-text transition-all"
            >
              Documentation
            </Link>
          </div>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 w-full mt-5 text-left">
            <div className="bg-surface p-8 rounded-2xl border border-border transition-all hover:-translate-y-1 hover:shadow-lg group">
                <div className="mb-4 text-primary bg-primary/10 w-fit p-3 rounded-xl group-hover:scale-110 transition-transform duration-300">
                    <Zap className="w-8 h-8" />
                </div>
                <h3 className="text-xl font-bold mb-3 text-text">Blazing Fast</h3>
                <p className="text-text-muted leading-relaxed">Up to 100x faster than traditional tools thanks to Rust's zero-cost abstractions.</p>
            </div>
             <div className="bg-surface p-8 rounded-2xl border border-border transition-all hover:-translate-y-1 hover:shadow-lg group">
                <div className="mb-4 text-primary bg-primary/10 w-fit p-3 rounded-xl group-hover:scale-110 transition-transform duration-300">
                    <Globe className="w-8 h-8" />
                </div>
                <h3 className="text-xl font-bold mb-3 text-text">WebAssembly</h3>
                <p className="text-text-muted leading-relaxed">Runs directly in your browser or Node.js edge environment with near-native performance.</p>
            </div>
             <div className="bg-surface p-8 rounded-2xl border border-border transition-all hover:-translate-y-1 hover:shadow-lg group">
                <div className="mb-4 text-primary bg-primary/10 w-fit p-3 rounded-xl group-hover:scale-110 transition-transform duration-300">
                    <Cpu className="w-8 h-8" />
                </div>
                <h3 className="text-xl font-bold mb-3 text-text">AST Based</h3>
                <p className="text-text-muted leading-relaxed">Safe, robust DOM mutations rather than regex tricks. Ensures 100% valid SVG output.</p>
            </div>
        </div>
      </div>
    </section>
  );
}
