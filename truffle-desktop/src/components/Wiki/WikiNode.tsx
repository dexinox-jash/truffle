// SPEC v2.0 SECTION 5.2.2: Wiki Node Detail Component
// Shows details of a selected wiki node

import React, { useState, useCallback } from 'react';
import { 
  Edit3, 
  Save, 
  X, 
  Calendar, 
  Link2, 
  Tag,
  Clock,
  Hash,
  MoreVertical
} from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { MilkdownEditor } from '../Editor/MilkdownEditor';

interface WikiNodeProps {
  nodeId: string;
  className?: string;
}

export const WikiNode: React.FC<WikiNodeProps> = ({ 
  nodeId,
  className = '' 
}) => {
  const { wikiNodes, updateWikiNode } = useAppStore();
  
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState('');
  const [editTitle, setEditTitle] = useState('');

  // Get node
  const node = wikiNodes.find(n => n.id === nodeId);

  // Start editing
  const handleStartEdit = useCallback(() => {
    if (!node) return;
    setEditContent(node.content);
    setEditTitle(node.title);
    setIsEditing(true);
  }, [node]);

  // Save edits
  const handleSave = useCallback(() => {
    if (!node) return;
    
    updateWikiNode(nodeId, {
      title: editTitle,
      content: editContent,
    });
    
    setIsEditing(false);
  }, [node, nodeId, editTitle, editContent, updateWikiNode]);

  // Cancel editing
  const handleCancel = useCallback(() => {
    setIsEditing(false);
    setEditContent('');
    setEditTitle('');
  }, []);

  if (!node) {
    return (
      <div className={`p-4 text-text-muted ${className}`}>
        <p>Node not found</p>
      </div>
    );
  }

  // Extract frontmatter
  const frontmatter = extractFrontmatter(node.content);
  const bodyContent = removeFrontmatter(node.content);

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Header */}
      <div className="flex items-start justify-between p-4 border-b border-border">
        <div className="flex-1 min-w-0">
          {isEditing ? (
            <input
              type="text"
              value={editTitle}
              onChange={(e) => setEditTitle(e.target.value)}
              className="w-full text-lg font-semibold bg-transparent border-b border-primary text-text-primary focus:outline-none"
              autoFocus
            />
          ) : (
            <h3 className="text-lg font-semibold text-text-primary truncate">
              {node.title}
            </h3>
          )}
          
          {/* Metadata */}
          <div className="flex items-center gap-3 mt-2 text-xs text-text-muted">
            <span className="flex items-center gap-1">
              <Calendar className="w-3 h-3" />
              {new Date(node.updatedAt).toLocaleDateString()}
            </span>
            <span className="flex items-center gap-1">
              <Link2 className="w-3 h-3" />
              {node.backlinkCount} backlinks
            </span>
            <span className="flex items-center gap-1 capitalize">
              <Tag className="w-3 h-3" />
              {node.nodeType}
            </span>
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1 ml-2">
          {isEditing ? (
            <>
              <button
                onClick={handleSave}
                className="p-1.5 rounded text-secondary hover:bg-secondary/10 transition-colors"
                title="Save"
              >
                <Save className="w-4 h-4" />
              </button>
              <button
                onClick={handleCancel}
                className="p-1.5 rounded text-text-muted hover:text-error hover:bg-error/10 transition-colors"
                title="Cancel"
              >
                <X className="w-4 h-4" />
              </button>
            </>
          ) : (
            <button
              onClick={handleStartEdit}
              className="p-1.5 rounded text-text-muted hover:text-text-primary hover:bg-surface-hover transition-colors"
              title="Edit"
            >
              <Edit3 className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {isEditing ? (
          <MilkdownEditor
            content={editContent}
            onChange={setEditContent}
            placeholder="Start writing..."
          />
        ) : (
          <div className="prose prose-invert prose-sm max-w-none">
            {/* Frontmatter display */}
            {Object.keys(frontmatter).length > 0 && (
              <div className="mb-4 p-3 bg-surface rounded-lg border border-border">
                <h4 className="text-xs font-medium text-text-secondary uppercase tracking-wide mb-2">
                  Metadata
                </h4>
                <dl className="grid grid-cols-2 gap-2 text-sm">
                  {Object.entries(frontmatter).map(([key, value]) => (
                    <div key={key}>
                      <dt className="text-text-muted capitalize">{key}:</dt>
                      <dd className="text-text-primary">{String(value)}</dd>
                    </div>
                  ))}
                </dl>
              </div>
            )}

            {/* Body content */}
            <div 
              className="wiki-content"
              dangerouslySetInnerHTML={{ 
                __html: renderWikiContent(bodyContent) 
              }}
            />
          </div>
        )}
      </div>
    </div>
  );
};

// Extract YAML frontmatter
function extractFrontmatter(content: string): Record<string, unknown> {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};
  
  const frontmatter: Record<string, unknown> = {};
  const lines = match[1].split('\n');
  
  lines.forEach(line => {
    const [key, ...valueParts] = line.split(':');
    if (key && valueParts.length > 0) {
      const value = valueParts.join(':').trim();
      frontmatter[key.trim()] = value;
    }
  });
  
  return frontmatter;
}

// Remove frontmatter from content
function removeFrontmatter(content: string): string {
  return content.replace(/^---\n[\s\S]*?\n---\n?/, '').trim();
}

// Render wiki content
function renderWikiContent(content: string): string {
  return content
    // Headers
    .replace(/^### (.*$)/gim, '<h3 class="text-lg font-semibold mt-4 mb-2">$1</h3>')
    .replace(/^## (.*$)/gim, '<h2 class="text-xl font-semibold mt-6 mb-3">$1</h2>')
    .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold mt-8 mb-4">$1</h1>')
    // Bold/Italic
    .replace(/\*\*\*(.*?)\*\*\*/g, '<strong><em>$1</em></strong>')
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    // WikiLinks
    .replace(/\[\[(.*?)\]\]/g, '<a href="#" class="text-primary hover:underline">[[$1]]</a>')
    // External links
    .replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2" class="text-secondary hover:underline" target="_blank" rel="noopener">$1</a>')
    // Lists
    .replace(/^- (.*$)/gim, '<li class="ml-4 list-disc">$1</li>')
    // Code
    .replace(/`([^`]+)`/g, '<code class="px-1 py-0.5 bg-surface rounded text-sm font-mono">$1</code>')
    // Blockquotes
    .replace(/^> (.*$)/gim, '<blockquote class="pl-4 border-l-2 border-primary italic text-text-secondary">$1</blockquote>')
    // Paragraphs
    .replace(/\n\n/g, '</p><p class="mb-4">')
    // Wrap in paragraph
    .replace(/^(.+)$/gim, '<p class="mb-4">$1</p>');
}

export default WikiNode;
