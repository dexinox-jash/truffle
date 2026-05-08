// SPEC v2.0 SECTION 5.2.1: Milkdown Markdown Editor
// WYSIWYM editor with custom WikiLink plugin

import React, { useEffect, useRef } from 'react';
import { 
  Milkdown, 
  useEditor, 
  MilkdownProvider 
} from '@milkdown/react';
import { 
  defaultValueCtx, 
  Editor, 
  rootCtx 
} from '@milkdown/core';
import { 
  commonmark, 
  paragraph, 
  heading, 
  blockquote,
  codeBlock,
  listItem,
  orderedList,
  bulletList,
  image,
  link,
  emphasis,
  strong,
  inlineCode,
  thematicBreak,
} from '@milkdown/preset-commonmark';
import { gfm } from '@milkdown/preset-gfm';
import { history } from '@milkdown/plugin-history';
import { listener, listenerCtx } from '@milkdown/plugin-listener';
import { tooltip } from '@milkdown/plugin-tooltip';
import { prism } from '@milkdown/plugin-prism';
import { WikiLinkPlugin } from './WikiLinkPlugin';

import 'prismjs/themes/prism-tomorrow.css';

interface MilkdownEditorProps {
  content?: string;
  onChange?: (content: string) => void;
  placeholder?: string;
  readOnly?: boolean;
  className?: string;
}

// Editor component
const EditorComponent: React.FC<MilkdownEditorProps> = ({
  content = '',
  onChange,
  placeholder = 'Start writing...',
  readOnly = false,
}) => {
  const editorRef = useRef<HTMLDivElement>(null);

  useEditor((root) => {
    const editor = Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, root);
        ctx.set(defaultValueCtx, content);
        
        // Set up listener for content changes
        if (onChange) {
          ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
            onChange(markdown);
          });
        }
      })
      .use(commonmark)
      .use(gfm)
      .use(history)
      .use(listener)
      .use(tooltip)
      .use(prism)
      .use(WikiLinkPlugin);

    return editor;
  }, [content]);

  return (
    <div 
      ref={editorRef}
      className="milkdown-editor prose prose-invert prose-sm max-w-none"
      data-placeholder={placeholder}
    />
  );
};

// Main component with provider
export const MilkdownEditor: React.FC<MilkdownEditorProps> = (props) => {
  return (
    <MilkdownProvider>
      <div className={`
        milkdown-wrapper relative
        ${props.className || ''}
      `}>
        <EditorComponent {...props} />
      </div>
    </MilkdownProvider>
  );
};

// Toolbar component for editor actions
interface EditorToolbarProps {
  onBold?: () => void;
  onItalic?: () => void;
  onHeading?: (level: number) => void;
  onLink?: () => void;
  onList?: (ordered: boolean) => void;
  onQuote?: () => void;
  onCode?: () => void;
  className?: string;
}

export const EditorToolbar: React.FC<EditorToolbarProps> = ({
  onBold,
  onItalic,
  onHeading,
  onLink,
  onList,
  onQuote,
  onCode,
  className = '',
}) => {
  const buttonClass = "p-1.5 rounded text-text-muted hover:text-text-primary hover:bg-surface-hover transition-colors";

  return (
    <div className={`
      flex items-center gap-1 p-2 bg-surface border-b border-border
      ${className}
    `}>
      {/* Text formatting */}
      <div className="flex items-center gap-0.5">
        <button onClick={onBold} className={buttonClass} title="Bold">
          <strong className="text-sm">B</strong>
        </button>
        <button onClick={onItalic} className={buttonClass} title="Italic">
          <em className="text-sm">I</em>
        </button>
      </div>

      <div className="w-px h-5 bg-border mx-1" />

      {/* Headings */}
      <div className="flex items-center gap-0.5">
        {[1, 2, 3].map((level) => (
          <button
            key={level}
            onClick={() => onHeading?.(level)}
            className={buttonClass}
            title={`Heading ${level}`}
          >
            <span className="text-sm font-bold">H{level}</span>
          </button>
        ))}
      </div>

      <div className="w-px h-5 bg-border mx-1" />

      {/* Lists */}
      <div className="flex items-center gap-0.5">
        <button onClick={() => onList?.(false)} className={buttonClass} title="Bullet list">
          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="8" y1="6" x2="21" y2="6" />
            <line x1="8" y1="12" x2="21" y2="12" />
            <line x1="8" y1="18" x2="21" y2="18" />
            <line x1="3" y1="6" x2="3.01" y2="6" />
            <line x1="3" y1="12" x2="3.01" y2="12" />
            <line x1="3" y1="18" x2="3.01" y2="18" />
          </svg>
        </button>
        <button onClick={() => onList?.(true)} className={buttonClass} title="Numbered list">
          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="10" y1="6" x2="21" y2="6" />
            <line x1="10" y1="12" x2="21" y2="12" />
            <line x1="10" y1="18" x2="21" y2="18" />
            <path d="M4 6h1v4" />
            <path d="M4 10h2" />
            <path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" />
          </svg>
        </button>
      </div>

      <div className="w-px h-5 bg-border mx-1" />

      {/* Other */}
      <div className="flex items-center gap-0.5">
        <button onClick={onLink} className={buttonClass} title="Link">
          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
          </svg>
        </button>
        <button onClick={onQuote} className={buttonClass} title="Quote">
          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z" />
            <path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3c0 1 0 1 1 1z" />
          </svg>
        </button>
        <button onClick={onCode} className={buttonClass} title="Code">
          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="16 18 22 12 16 6" />
            <polyline points="8 6 2 12 8 18" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default MilkdownEditor;
