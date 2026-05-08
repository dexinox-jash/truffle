// SPEC v2.0 SECTION 5.2.2: Backlinks Component
// Shows nodes that link to the current node

import React from 'react';
import { ArrowLeft, FileText, ExternalLink } from 'lucide-react';
import { useAppStore } from '../../store/appStore';

interface BacklinksProps {
  nodeId: string;
  className?: string;
}

interface Backlink {
  id: string;
  title: string;
  excerpt: string;
  context: string;
}

export const Backlinks: React.FC<BacklinksProps> = ({ 
  nodeId,
  className = '' 
}) => {
  const { wikiNodes, selectNode } = useAppStore();

  // Get current node
  const currentNode = wikiNodes.find(n => n.id === nodeId);

  // Mock backlinks - would come from API
  const backlinks: Backlink[] = React.useMemo(() => {
    // Find nodes that reference this node in their content
    return wikiNodes
      .filter(n => n.id !== nodeId && n.content.includes(currentNode?.title || ''))
      .map(n => ({
        id: n.id,
        title: n.title,
        excerpt: n.excerpt || n.content.slice(0, 100) + '...',
        context: `Mentions "${currentNode?.title}"`,
      }))
      .slice(0, 10);
  }, [wikiNodes, nodeId, currentNode]);

  if (!currentNode) return null;

  return (
    <div className={`border-t border-border ${className}`}>
      {/* Header */}
      <div className="flex items-center gap-2 px-4 py-3 bg-surface/30">
        <ArrowLeft className="w-4 h-4 text-text-muted" />
        <h4 className="text-sm font-medium text-text-secondary">
          Backlinks
        </h4>
        <span className="text-xs text-text-muted">
          ({backlinks.length})
        </span>
      </div>

      {/* Backlink List */}
      <div className="max-h-64 overflow-auto">
        {backlinks.length === 0 ? (
          <div className="px-4 py-6 text-center text-text-muted">
            <p className="text-sm">No backlinks yet</p>
            <p className="text-xs mt-1">
              Other nodes linking to this will appear here
            </p>
          </div>
        ) : (
          <div className="divide-y divide-border">
            {backlinks.map((backlink) => (
              <div
                key={backlink.id}
                onClick={() => selectNode(backlink.id)}
                className="group px-4 py-3 cursor-pointer hover:bg-surface-hover transition-colors"
              >
                <div className="flex items-start gap-2">
                  <FileText className="w-4 h-4 text-text-muted mt-0.5 flex-shrink-0" />
                  
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <h5 className="text-sm font-medium text-text-primary group-hover:text-primary transition-colors">
                        {backlink.title}
                      </h5>
                      <ExternalLink className="w-3 h-3 text-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
                    </div>
                    
                    <p className="text-xs text-text-muted mt-1 line-clamp-2">
                      {backlink.excerpt}
                    </p>
                    
                    <p className="text-xs text-primary/70 mt-1">
                      {backlink.context}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer - Show all if more than 10 */}
      {currentNode.backlinkCount > 10 && (
        <div className="px-4 py-2 border-t border-border">
          <button className="text-xs text-primary hover:underline">
            Show all {currentNode.backlinkCount} backlinks
          </button>
        </div>
      )}
    </div>
  );
};

export default Backlinks;
