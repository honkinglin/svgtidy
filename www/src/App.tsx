import { Suspense } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Header } from './components/Header';
import { Hero } from './components/Hero';
import { Playground } from './components/Playground';
import { Docs } from './components/Docs';

function Home() {
    return (
        <main>
            <Hero />
            <Suspense fallback={<div className="container" style={{padding: '40px', textAlign: 'center'}}>Loading WASM module...</div>}>
              <Playground />
            </Suspense>
        </main>
    );
}

function App() {
  return (
    <BrowserRouter>
      <Header />
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/docs" element={<Docs />} />
      </Routes>
      <footer style={{
        textAlign: 'center', 
        padding: '40px', 
        color: 'var(--color-text-muted)',
        borderTop: '1px solid var(--color-border)',
        marginTop: 'auto'
      }}>
        <div className="container">
          <p>© {new Date().getFullYear()} SvgTidy. MIT License.</p>
        </div>
      </footer>
    </BrowserRouter>
  )
}

export default App
