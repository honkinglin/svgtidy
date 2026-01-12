import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';


// Import raw README content via Vite
import readmeContent from '../../../README.md?raw';

export function Docs() {
// In a real app we might fetch this, but bundling it is fine for now
  const content = readmeContent;

  return (
    <div className="container mx-auto px-5 py-12 max-w-4xl">
      <div className="prose prose-lg prose-slate dark:prose-invert max-w-none">
        <ReactMarkdown 
            remarkPlugins={[remarkGfm]}
            components={{
                img: ({...props}) => {
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
