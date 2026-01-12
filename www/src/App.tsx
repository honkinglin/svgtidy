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
      <footer className="text-center py-10 text-text-muted border-t border-border mt-auto">
        <div className="container mx-auto px-5">
          <p>© {new Date().getFullYear()} SvgTidy. MIT License.</p>
        </div>
      </footer>
    </BrowserRouter>
  )
}

export default App
