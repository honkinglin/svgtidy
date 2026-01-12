import { Github, Moon, Sun } from 'lucide-react';
import { useEffect, useState } from 'react';

export function Header() {
  const [isDark, setIsDark] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('theme') === 'dark' || 
        (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches);
    }
    return false;
  });

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }, [isDark]);

  return (
    <header className="h-16 border-b border-border bg-surface sticky top-0 z-50">
      <div className="container mx-auto px-5 h-full flex items-center justify-between">
        <a href="/" className="flex items-center gap-3 text-inherit no-underline">
          <img src="/logo.svg" alt="SvgTidy Logo" className="h-8" />
        </a>
        <div className="flex items-center gap-4">
            <button
              onClick={() => setIsDark(!isDark)}
              className="p-2 text-text-muted hover:text-text rounded-full hover:bg-black/5 transition-colors"
              title={isDark ? "Switch to light mode" : "Switch to dark mode"}
            >
              {isDark ? <Sun size={20} /> : <Moon size={20} />}
            </button>
            <a 
              href="https://github.com/honkinglin/svgtidy" 
              target="_blank" 
              rel="noreferrer"
              className="text-text-muted hover:text-text transition-colors duration-200"
            >
              <Github size={24} />
            </a>
        </div>
      </div>
    </header>
  );
}
