import { Link } from 'react-router-dom';
import { ArrowRight, Zap, Globe, Cpu } from 'lucide-react';
import './Hero.css';

export function Hero() {
  const scrollToPlayground = () => {
    document.getElementById('playground')?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <section className="hero">
      <div className="container hero-inner">
        <div className="hero-content">
          <h1 className="hero-title">
            Optimize your SVGs <br/>
            <span className="highlight">lightning fast.</span>
          </h1>
          <p className="hero-subtitle">
            A high-performance SVG optimizer written in Rust. 
            Available as a CLI, generic library, and WebAssembly module.
          </p>
          <div className="hero-actions">
            <button className="btn btn-primary" onClick={scrollToPlayground}>
              Try Online <ArrowRight size={18} />
            </button>
            <Link to="/docs" className="btn btn-outline">
              Documentation
            </Link>
          </div>
        </div>
        
        <div className="features-grid">
            <div className="feature-card">
                <Zap className="feature-icon" />
                <h3>Blazing Fast</h3>
                <p>Up to 100x faster than traditional tools thanks to Rust.</p>
            </div>
             <div className="feature-card">
                <Globe className="feature-icon" />
                <h3>WebAssembly</h3>
                <p>Runs directly in your browser or Node.js edge environment.</p>
            </div>
             <div className="feature-card">
                <Cpu className="feature-icon" />
                <h3>AST Based</h3>
                <p>Safe, robust DOM mutations rather than regex tricks.</p>
            </div>
        </div>
      </div>
    </section>
  );
}
