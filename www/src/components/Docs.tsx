import rehypeSlug from 'rehype-slug';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import { useEffect, useState, useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

// Import documentation content
import docsContent from '../content/docs.md?raw';

export function Docs() {
  const content = docsContent;
  const [activeId, setActiveId] = useState<string>('');

  // Extract headings for TOC
  const headings = useMemo(() => content.match(/^#{2,3} .+$/gm)?.map(heading => {
    const level = heading.match(/^#+/)?.[0].length || 2;
    const text = heading.replace(/^#+ /, '');
    const id = text.toLowerCase().replace(/[^\w]+/g, '-');
    return { level, text, id };
  }) || [], [content]);

  useEffect(() => {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          setActiveId(entry.target.id);
        }
      });
    }, { rootMargin: '-100px 0px -66%' });

    headings.forEach(({ id }) => {
      const element = document.getElementById(id);
      if (element) observer.observe(element);
    });

    return () => observer.disconnect();
  }, [headings]);

  return (
    <div className="container mx-auto px-5 py-12 max-w-6xl">
      <div className="grid grid-cols-1 lg:grid-cols-[1fr_250px] gap-12">
        {/* Main Content */}
        <div className="prose prose-lg dark:prose-invert prose-headings:font-bold prose-headings:tracking-tight prose-a:text-primary hover:prose-a:text-primary-dark max-w-none">
          <ReactMarkdown 
            remarkPlugins={[remarkGfm]}
            rehypePlugins={[rehypeSlug, rehypeAutolinkHeadings]}
            components={{
                img: ({...props}) => {
                    return <img {...props} alt={props.alt || ''} className="rounded-lg border border-border shadow-sm my-8" style={{maxWidth: '100%'}} />;
                },
                code: ({className, children, ...props}: React.HTMLAttributes<HTMLElement>) => {
                    const match = /language-(\w+)/.exec(className || '');
                    return match ? (
                        <code className={className} {...props}>
                            {children}
                        </code>
                    ) : (
                        <code className="bg-black/5 dark:bg-white/10 px-1.5 py-0.5 rounded font-mono text-sm before:content-[''] after:content-['']" {...props}>
                            {children}
                        </code>
                    )
                },
                pre: ({children}: React.HTMLAttributes<HTMLPreElement>) => {
                   return <pre className="bg-slate-900 text-slate-50 border border-slate-700 rounded-xl p-4 overflow-x-auto shadow-sm my-6">{children}</pre>;
                },
                blockquote: ({children}: React.HTMLAttributes<HTMLQuoteElement>) => {
                    return <blockquote className="border-l-4 border-primary pl-4 text-text-muted italic my-6 bg-primary/5 py-2 rounded-r">{children}</blockquote>
                }
            }}
          >
            {content}
          </ReactMarkdown>
        </div>

        {/* Sidebar TOC */}
        <aside className="hidden lg:block">
          <div className="sticky top-24 max-h-[calc(100vh-8rem)] overflow-y-auto">
            <h4 className="font-bold mb-4 text-sm text-text-muted uppercase tracking-wider">On this page</h4>
            <nav className="flex flex-col gap-1 border-l border-border pl-4">
              {headings.map(({ level, text, id }) => (
                <a
                  key={id}
                  href={`#${id}`}
                  className={`text-sm py-1 transition-colors border-l -ml-4 pl-4 ${
                    activeId === id 
                      ? 'text-primary font-medium border-primary' 
                      : 'text-text-muted hover:text-text border-transparent hover:border-text-muted'
                  } ${level === 3 ? 'pl-8' : ''}`}
                  onClick={(e) => {
                      e.preventDefault();
                      document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' });
                  }}
                >
                  {text}
                </a>
              ))}
            </nav>
          </div>
        </aside>
      </div>
    </div>
  );
}
