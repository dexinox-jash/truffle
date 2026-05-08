// SPEC v2.0 SECTION 5.2.2: Before/After Diff Component
// Shows comparison between raw artifact and compiled wiki content

import React, { useState } from 'react';
import { Split, Eye, Code, FileText } from 'lucide-react';

interface BeforeAfterDiffProps {
  artifactId: string | null;
  className?: string;
}

export const BeforeAfterDiff: React.FC<BeforeAfterDiffProps> = ({ 
  artifactId,
  className = '' 
}) => {
  const [viewMode, setViewMode] = useState<'split' | 'before' | 'after'>('split');
  const [showRaw, setShowRaw] = useState(false);

  if (!artifactId) {
    return (
      <div className={`flex items-center justify-center h-full text-text-muted ${className}`}>
        <p>Select an artifact to view diff</p>
      </div>
    );
  }

  // Mock data - would come from API
  const beforeContent = {
    filename: 'screenshot_2024-01-15.png',
    ocrText: 'Receipt from Coffee Shop\nTotal: $12.50\nDate: 2024-01-15',
    metadata: {
      size: '2.4 MB',
      dimensions: '1920x1080',
      capturedAt: '2024-01-15T10:30:00Z',
    },
  };

  const afterContent = `---
title: "Coffee Shop Receipt"
date: 2024-01-15
type: receipt
merchant: "Coffee Shop"
total: 12.50
currency: USD
---

# Coffee Shop Receipt

**Merchant:** Coffee Shop  
**Date:** January 15, 2024  
**Total:** $12.50

## Items

- Coffee - $4.50
- Pastry - $5.00
- Tip - $3.00

## Related

- [[Expenses/2024-01]]
- [[Coffee Shop]]
`;

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-surface/50">
        <div className="flex items-center gap-1">
          <button
            onClick={() => setViewMode('split')}
            className={`
              p-1.5 rounded transition-colors
              ${viewMode === 'split' 
                ? 'bg-surface-elevated text-text-primary' 
                : 'text-text-secondary hover:text-text-primary'
              }
            `}
            title="Split view"
          >
            <Split className="w-4 h-4" />
          </button>
          <button
            onClick={() => setViewMode('before')}
            className={`
              p-1.5 rounded transition-colors
              ${viewMode === 'before' 
                ? 'bg-surface-elevated text-text-primary' 
                : 'text-text-secondary hover:text-text-primary'
              }
            `}
            title="Before only"
          >
            <Eye className="w-4 h-4" />
          </button>
          <button
            onClick={() => setViewMode('after')}
            className={`
              p-1.5 rounded transition-colors
              ${viewMode === 'after' 
                ? 'bg-surface-elevated text-text-primary' 
                : 'text-text-secondary hover:text-text-primary'
              }
            `}
            title="After only"
          >
            <FileText className="w-4 h-4" />
          </button>
        </div>

        <button
          onClick={() => setShowRaw(!showRaw)}
          className={`
            flex items-center gap-1.5 px-2 py-1 rounded text-xs
            transition-colors
            ${showRaw 
              ? 'bg-primary/10 text-primary' 
              : 'text-text-secondary hover:text-text-primary'
            }
          `}
        >
          <Code className="w-3.5 h-3.5" />
          <span>Raw</span>
        </button>
      </div>

      {/* Diff Content */}
      <div className="flex-1 overflow-hidden">
        <div 
          className={`
            flex h-full
            ${viewMode === 'split' ? 'flex-row' : 'flex-col'}
          `}
        >
          {/* Before Panel */}
          {(viewMode === 'split' || viewMode === 'before') && (
            <div 
              className={`
                ${viewMode === 'split' ? 'w-1/2 border-r border-border' : 'flex-1'}
                overflow-auto p-4
              `}
            >
              <div className="flex items-center gap-2 mb-3">
                <span className="text-xs font-medium text-text-muted uppercase tracking-wide">
                  Before (Raw)
                </span>
              </div>

              <div className="space-y-4">
                {/* Image Preview */}
                <div className="aspect-video bg-surface rounded-lg border border-border overflow-hidden">
                  <div className="w-full h-full flex items-center justify-center bg-surface-elevated">
                    <span className="text-4xl">📷</span>
                  </div>
                </div>

                {/* OCR Text */}
                <div className="bg-surface rounded-lg border border-border p-3">
                  <h4 className="text-xs font-medium text-text-secondary mb-2">
                    Extracted Text (OCR)
                  </h4>
                  <pre className="text-sm text-text-primary whitespace-pre-wrap font-mono">
                    {beforeContent.ocrText}
                  </pre>
                </div>

                {/* Metadata */}
                <div className="bg-surface rounded-lg border border-border p-3">
                  <h4 className="text-xs font-medium text-text-secondary mb-2">
                    Metadata
                  </h4>
                  <dl className="space-y-1 text-sm">
                    <div className="flex justify-between">
                      <dt className="text-text-muted">Size:</dt>
                      <dd className="text-text-primary">{beforeContent.metadata.size}</dd>
                    </div>
                    <div className="flex justify-between">
                      <dt className="text-text-muted">Dimensions:</dt>
                      <dd className="text-text-primary">{beforeContent.metadata.dimensions}</dd>
                    </div>
                    <div className="flex justify-between">
                      <dt className="text-text-muted">Captured:</dt>
                      <dd className="text-text-primary">
                        {new Date(beforeContent.metadata.capturedAt).toLocaleString()}
                      </dd>
                    </div>
                  </dl>
                </div>
              </div>
            </div>
          )}

          {/* After Panel */}
          {(viewMode === 'split' || viewMode === 'after') && (
            <div 
              className={`
                ${viewMode === 'split' ? 'w-1/2' : 'flex-1'}
                overflow-auto p-4
              `}
            >
              <div className="flex items-center gap-2 mb-3">
                <span className="text-xs font-medium text-secondary uppercase tracking-wide">
                  After (Compiled)
                </span>
              </div>

              {showRaw ? (
                <pre className="text-sm font-mono text-text-primary whitespace-pre-wrap bg-surface rounded-lg border border-border p-4">
                  {afterContent}
                </pre>
              ) : (
                <div className="prose prose-invert prose-sm max-w-none">
                  <div className="bg-surface rounded-lg border border-border p-4">
                    {/* Rendered Markdown */}
                    <div dangerouslySetInnerHTML={{ 
                      __html: renderMarkdown(afterContent) 
                    }} />
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

// Simple markdown renderer (would use proper library in production)
function renderMarkdown(content: string): string {
  return content
    .replace(/---[\s\S]*?---/, '') // Remove frontmatter
    .replace(/^# (.*$)/gm, '<h1 class="text-xl font-bold mb-4">$1</h1>')
    .replace(/^## (.*$)/gm, '<h2 class="text-lg font-semibold mb-3 mt-4">$1</h2>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\[\[(.*?)\]\]/g, '<a href="#" class="text-primary hover:underline">[[$1]]</a>')
    .replace(/^- (.*$)/gm, '<li class="ml-4">$1</li>')
    .replace(/\n/g, '<br />');
}

export default BeforeAfterDiff;
