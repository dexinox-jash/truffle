// SPEC v2.0 SECTION 5.2.2: Obsidian-style Tree View Component
// Hierarchical tree view for wiki nodes

import React, { useState, useCallback } from 'react';
import { 
  ChevronRight, 
  ChevronDown, 
  FileText, 
  Folder,
  FolderOpen,
  Hash,
  Calendar,
  User,
  MoreHorizontal
} from 'lucide-react';

interface WikiNode {
  id: string;
  title: string;
  content: string;
  nodeType: 'entity' | 'concept' | 'chronology' | 'index';
  updatedAt: string;
  backlinkCount: number;
  forwardLinkCount: number;
}

interface TreeViewProps {
  nodes: WikiNode[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  groupedNodes: Record<string, WikiNode[]>;
  className?: string;
}

interface TreeNodeProps {
  node: WikiNode;
  isSelected: boolean;
  onSelect: (id: string) => void;
  depth?: number;
}

// Get icon for node type
const getNodeTypeIcon = (type: string, isOpen?: boolean) => {
  switch (type) {
    case 'entity':
      return <User className="w-4 h-4" />;
    case 'concept':
      return <Hash className="w-4 h-4" />;
    case 'chronology':
      return <Calendar className="w-4 h-4" />;
    case 'index':
      return isOpen ? <FolderOpen className="w-4 h-4" /> : <Folder className="w-4 h-4" />;
    default:
      return <FileText className="w-4 h-4" />;
  }
};

// Get color for node type
const getNodeTypeColor = (type: string) => {
  switch (type) {
    case 'entity':
      return 'text-blue-400';
    case 'concept':
      return 'text-purple-400';
    case 'chronology':
      return 'text-green-400';
    case 'index':
      return 'text-yellow-400';
    default:
      return 'text-text-muted';
  }
};

// Individual tree node
const TreeNode: React.FC<TreeNodeProps> = ({ 
  node, 
  isSelected, 
  onSelect,
  depth = 0 
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const hasChildren = node.forwardLinkCount > 0;

  const handleToggle = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(!isExpanded);
  }, [isExpanded]);

  const handleSelect = useCallback(() => {
    onSelect(node.id);
  }, [node.id, onSelect]);

  return (
    <div className="select-none">
      <div
        onClick={handleSelect}
        className={`
          group flex items-center gap-1 py-1.5 pr-2 cursor-pointer
          transition-colors
          ${isSelected 
            ? 'bg-primary/10' 
            : 'hover:bg-surface-hover'
          }
        `}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
      >
        {/* Expand/Collapse */}
        {hasChildren ? (
          <button
            onClick={handleToggle}
            className="p-0.5 rounded hover:bg-surface-hover text-text-muted"
          >
            {isExpanded ? (
              <ChevronDown className="w-3.5 h-3.5" />
            ) : (
              <ChevronRight className="w-3.5 h-3.5" />
            )}
          </button>
        ) : (
          <span className="w-5" />
        )}

        {/* Icon */}
        <span className={getNodeTypeColor(node.nodeType)}>
          {getNodeTypeIcon(node.nodeType, isExpanded)}
        </span>

        {/* Title */}
        <span className={`
          flex-1 text-sm truncate
          ${isSelected ? 'text-primary font-medium' : 'text-text-primary'}
        `}>
          {node.title}
        </span>

        {/* Backlink count */}
        {node.backlinkCount > 0 && (
          <span className="text-xs text-text-muted">
            {node.backlinkCount}
          </span>
        )}

        {/* Context menu */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            // Show context menu
          }}
          className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-surface-hover text-text-muted"
        >
          <MoreHorizontal className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Children - placeholder for now */}
      {isExpanded && hasChildren && (
        <div className="tree-item-enter">
          {/* Would render child nodes here */}
          <div 
            className="py-1 pl-8 text-xs text-text-muted"
            style={{ paddingLeft: `${(depth + 1) * 16 + 8}px` }}
          >
            {node.forwardLinkCount} linked nodes...
          </div>
        </div>
      )}
    </div>
  );
};

// Main tree view
export const TreeView: React.FC<TreeViewProps> = ({
  nodes,
  selectedId,
  onSelect,
  groupedNodes,
  className = '',
}) => {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    new Set(['entity', 'concept', 'chronology', 'index'])
  );

  const toggleGroup = useCallback((group: string) => {
    setExpandedGroups(prev => {
      const next = new Set(prev);
      if (next.has(group)) {
        next.delete(group);
      } else {
        next.add(group);
      }
      return next;
    });
  }, []);

  const groupLabels: Record<string, string> = {
    entity: 'Entities',
    concept: 'Concepts',
    chronology: 'Chronology',
    index: 'Indexes',
  };

  return (
    <div className={`overflow-auto h-full ${className}`}>
      {/* Root Nodes (ungrouped) */}
      {nodes.filter(n => !n.nodeType).map(node => (
        <TreeNode
          key={node.id}
          node={node}
          isSelected={selectedId === node.id}
          onSelect={onSelect}
        />
      ))}

      {/* Grouped Nodes */}
      {Object.entries(groupedNodes).map(([type, typeNodes]) => {
        if (typeNodes.length === 0) return null;
        
        const isExpanded = expandedGroups.has(type);
        
        return (
          <div key={type} className="select-none">
            {/* Group Header */}
            <div
              onClick={() => toggleGroup(type)}
              className="group flex items-center gap-1 py-2 px-2 cursor-pointer hover:bg-surface-hover transition-colors sticky top-0 bg-background z-10"
            >
              <button className="p-0.5 rounded hover:bg-surface-hover text-text-muted">
                {isExpanded ? (
                  <ChevronDown className="w-4 h-4" />
                ) : (
                  <ChevronRight className="w-4 h-4" />
                )}
              </button>
              
              <span className={getNodeTypeColor(type)}>
                {getNodeTypeIcon(type, isExpanded)}
              </span>
              
              <span className="text-sm font-medium text-text-secondary">
                {groupLabels[type]}
              </span>
              
              <span className="text-xs text-text-muted ml-1">
                ({typeNodes.length})
              </span>
            </div>

            {/* Group Content */}
            {isExpanded && (
              <div className="tree-item-enter">
                {typeNodes.map(node => (
                  <TreeNode
                    key={node.id}
                    node={node}
                    isSelected={selectedId === node.id}
                    onSelect={onSelect}
                    depth={1}
                  />
                ))}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
};

export default TreeView;
