// SPEC v2.0 SECTION 5.2.2: Wiki Navigator Panel
// Right panel showing wiki nodes in Obsidian-style tree view

import React, { useState, useCallback } from 'react';
import { 
  FileText, 
  Plus, 
  Search, 
  FolderTree,
  ChevronRight,
  ChevronDown,
  MoreHorizontal,
  Edit3,
  Trash2,
  ExternalLink
} from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { TreeView } from './TreeView';
import { WikiNode } from './WikiNode';
import { Backlinks } from './Backlinks';

interface WikiPanelProps {
  className?: string;
}

type ViewType = 'tree' | 'list' | 'recent';

export const WikiPanel: React.FC<WikiPanelProps> = ({ className = '' }) => {
  const {
    wikiNodes,
    selectedNodeId,
    selectNode,
  } = useAppStore();

  const [viewType, setViewType] = useState<ViewType>('tree');
  const [searchQuery, setSearchQuery] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newNodeTitle, setNewNodeTitle] = useState('');

  // Filter nodes by search
  const filteredNodes = React.useMemo(() => {
    if (!searchQuery.trim()) return wikiNodes;
    
    const query = searchQuery.toLowerCase();
    return wikiNodes.filter(node => 
      node.title.toLowerCase().includes(query) ||
      node.content.toLowerCase().includes(query)
    );
  }, [wikiNodes, searchQuery]);

  // Group nodes by type for tree view
  const groupedNodes = React.useMemo(() => {
    const groups: Record<string, typeof wikiNodes> = {
      entity: [],
      concept: [],
      chronology: [],
      index: [],
    };
    
    filteredNodes.forEach(node => {
      groups[node.nodeType]?.push(node);
    });
    
    return groups;
  }, [filteredNodes]);

  // Handle create node
  const handleCreateNode = useCallback(() => {
    if (!newNodeTitle.trim()) return;
    
    // TODO: Create node via Tauri API
    console.log('Create node:', newNodeTitle);
    
    setNewNodeTitle('');
    setIsCreating(false);
  }, [newNodeTitle]);

  // Handle node delete
  const handleDeleteNode = useCallback((id: string) => {
    // TODO: Delete node via Tauri API
    console.log('Delete node:', id);
  }, []);

  return (
    <div className={`flex flex-col h-full bg-background ${className}`}>
      {/* Panel Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <FileText className="w-4 h-4 text-secondary" />
          <h2 className="text-sm font-semibold text-text-primary">Wiki Navigator</h2>
          <span className="text-xs text-text-muted">({wikiNodes.length})</span>
        </div>
        
        <div className="flex items-center gap-1">
          {/* View Toggle */}
          <div className="flex items-center bg-surface rounded-md p-0.5">
            <button
              onClick={() => setViewType('tree')}
              className={`
                p-1 rounded transition-colors
                ${viewType === 'tree' 
                  ? 'bg-surface-elevated text-text-primary' 
                  : 'text-text-secondary hover:text-text-primary'
                }
              `}
              title="Tree view"
            >
              <FolderTree className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewType('list')}
              className={`
                p-1 rounded transition-colors
                ${viewType === 'list' 
                  ? 'bg-surface-elevated text-text-primary' 
                  : 'text-text-secondary hover:text-text-primary'
                }
              `}
              title="List view"
            >
              <FileText className="w-4 h-4" />
            </button>
          </div>
          
          {/* Create Button */}
          <button
            onClick={() => setIsCreating(true)}
            className="flex items-center gap-1.5 px-2 py-1.5 bg-secondary text-background rounded-md text-xs font-medium hover:bg-secondary-hover transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">New</span>
          </button>
        </div>
      </div>

      {/* Search */}
      <div className="px-3 py-2 border-b border-border">
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" />
          <input
            type="text"
            placeholder="Search wiki..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-3 py-1.5 bg-surface border border-border rounded-md text-sm text-text-primary placeholder:text-text-muted focus:border-primary focus:ring-1 focus:ring-primary/20 transition-all"
          />
        </div>
      </div>

      {/* Create Node Input */}
      {isCreating && (
        <div className="px-3 py-2 border-b border-border bg-surface/50">
          <div className="flex items-center gap-2">
            <input
              type="text"
              placeholder="Node title..."
              value={newNodeTitle}
              onChange={(e) => setNewNodeTitle(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreateNode();
                if (e.key === 'Escape') setIsCreating(false);
              }}
              autoFocus
              className="flex-1 px-3 py-1.5 bg-background border border-border rounded-md text-sm text-text-primary placeholder:text-text-muted focus:border-primary focus:ring-1 focus:ring-primary/20 transition-all"
            />
            <button
              onClick={handleCreateNode}
              className="px-3 py-1.5 bg-primary text-background rounded-md text-xs font-medium hover:bg-primary-hover transition-colors"
            >
              Create
            </button>
            <button
              onClick={() => setIsCreating(false)}
              className="px-3 py-1.5 text-text-secondary hover:text-text-primary transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-hidden flex">
        {/* Node List */}
        <div className="flex-1 overflow-auto">
          {filteredNodes.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-text-muted">
              <div className="w-16 h-16 rounded-full bg-surface flex items-center justify-center mb-4">
                <FileText className="w-8 h-8" />
              </div>
              <p className="text-sm">
                {searchQuery ? 'No matching nodes' : 'No wiki nodes yet'}
              </p>
              <p className="text-xs mt-1">
                {searchQuery 
                  ? 'Try a different search' 
                  : 'Compile screenshots to create nodes'
                }
              </p>
            </div>
          ) : viewType === 'tree' ? (
            <TreeView 
              nodes={filteredNodes}
              selectedId={selectedNodeId}
              onSelect={selectNode}
              groupedNodes={groupedNodes}
            />
          ) : (
            <div className="p-2 space-y-1">
              {filteredNodes.map((node) => (
                <div
                  key={node.id}
                  onClick={() => selectNode(node.id)}
                  className={`
                    group flex items-center gap-2 p-2 rounded-lg cursor-pointer
                    transition-colors
                    ${selectedNodeId === node.id 
                      ? 'bg-primary/10 border border-primary/30' 
                      : 'hover:bg-surface-hover border border-transparent'
                    }
                  `}
                >
                  <FileText className={`
                    w-4 h-4 flex-shrink-0
                    ${selectedNodeId === node.id ? 'text-primary' : 'text-text-muted'}
                  `} />
                  
                  <div className="flex-1 min-w-0">
                    <p className={`
                      text-sm font-medium truncate
                      ${selectedNodeId === node.id ? 'text-primary' : 'text-text-primary'}
                    `}>
                      {node.title}
                    </p>
                    <p className="text-xs text-text-muted truncate">
                      {node.excerpt || 'No content'}
                    </p>
                  </div>
                  
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        // Edit
                      }}
                      className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-surface-hover"
                    >
                      <Edit3 className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteNode(node.id);
                      }}
                      className="p-1 rounded text-text-muted hover:text-error hover:bg-error/10"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Selected Node Details */}
        {selectedNodeId && (
          <div className="w-80 border-l border-border bg-surface/30 overflow-auto">
            <WikiNode nodeId={selectedNodeId} />
            <Backlinks nodeId={selectedNodeId} />
          </div>
        )}
      </div>
    </div>
  );
};

export default WikiPanel;
