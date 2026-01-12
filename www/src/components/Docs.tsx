import { useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import './Docs.css';

// Import raw README content via Vite
import readmeContent from '../../../README.md?raw';

export function Docs() {
  const [content, setContent] = useState('');

  useEffect(() => {
    // In a real app we might fetch this, but bundling it is fine for now
    setContent(readmeContent);
  }, []);

  return (
    <div className="container docs-container">
      <div className="docs-content">
        <ReactMarkdown 
            remarkPlugins={[remarkGfm]}
            components={{
                img: ({node, ...props}) => {
                    // Fix relative image paths if necessary, though logo.svg is in public/
                    // If README uses relative paths like ./logo.svg, they might break.
                    // For now, assume simpler images or absolute URLs.
                    return <img {...props} style={{maxWidth: '100%'}} />;
                }
            }}
        >
            {content}
        </ReactMarkdown>
      </div>
    </div>
  );
}
