// SPEC v2.0 SECTION 5.2.2: Command Palette Component
// Cmd+K interface for quick actions

import React, { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { 
  Command, 
  X, 
  Search,
  Sparkles,
  FileDown,
  Settings,
  RefreshCw,
  Image,
  BookOpen,
  Columns,
  Zap,
  HelpCircle,
  Keyboard,
  Moon,
  Sun,
  LogOut,
  Cloud,
  QrCode
} from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { useSync } from '../../store/syncStore';

interface CommandPaletteProps {
  onClose: () => void;
  className?: string;
}

interface CommandItem {
  id: string;
  title: string;
  description?: string;
  icon: React.ReactNode;
  shortcut?: string;
  keywords: string[];
  action: () => void;
  section: string;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ 
  onClose,
  className = '' 
}) => {
  const {
    viewMode,
    setViewMode,
    isCompiling,
    compilationQueue,
    artifacts,
    setIsCompiling,
    openSearch,
  } = useAppStore();

  const { startPairing } = useSync();
  
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Define all commands
  const commands: CommandItem[] = useMemo(() => [
    // View Commands
    {
      id: 'view-raw',
      title: 'Switch to Raw View',
      description: 'Show only raw artifacts panel',
      icon: <Image className="w-4 h-4" />,
      shortcut: '⌘1',
      keywords: ['view', 'raw', 'artifacts', 'screenshots'],
      action: () => {
        setViewMode('raw');
        onClose();
      },
      section: 'View',
    },
    {
      id: 'view-split',
      title: 'Switch to Split View',
      description: 'Show all three panels',
      icon: <Columns className="w-4 h-4" />,
      shortcut: '⌘2',
      keywords: ['view', 'split', 'all', 'panels'],
      action: () => {
        setViewMode('split');
        onClose();
      },
      section: 'View',
    },
    {
      id: 'view-wiki',
      title: 'Switch to Wiki View',
      description: 'Show only wiki navigator',
      icon: <BookOpen className="w-4 h-4" />,
      shortcut: '⌘3',
      keywords: ['view', 'wiki', 'nodes', 'navigator'],
      action: () => {
        setViewMode('wiki');
        onClose();
      },
      section: 'View',
    },
    
    // Compilation Commands
    {
      id: 'compile-all',
      title: 'Compile All Pending',
      description: `Compile ${artifacts.filter(a => a.status === 'pending').length} pending artifacts`,
      icon: <Sparkles className="w-4 h-4" />,
      keywords: ['compile', 'all', 'pending', 'process'],
      action: () => {
        // TODO: Trigger batch compilation
        console.log('Compile all pending');
        onClose();
      },
      section: 'Compilation',
    },
    {
      id: 'compile-cancel',
      title: 'Cancel Compilation',
      description: 'Stop current compilation process',
      icon: <X className="w-4 h-4" />,
      keywords: ['compile', 'cancel', 'stop', 'abort'],
      action: () => {
        setIsCompiling(false);
        onClose();
      },
      section: 'Compilation',
      
    },
    
    // Search Commands
    {
      id: 'search',
      title: 'Search Everything',
      description: 'Search across all content',
      icon: <Search className="w-4 h-4" />,
      shortcut: '⌘⇧F',
      keywords: ['search', 'find', 'query'],
      action: () => {
        onClose();
        openSearch();
      },
      section: 'Search',
    },
    
    // Sync Commands
    {
      id: 'sync-now',
      title: 'Sync Now',
      description: 'Trigger manual sync',
      icon: <RefreshCw className="w-4 h-4" />,
      keywords: ['sync', 'synchronize', 'update'],
      action: () => {
        // TODO: Trigger sync
        console.log('Sync now');
        onClose();
      },
      section: 'Sync',
    },
    {
      id: 'pair-device',
      title: 'Pair New Device',
      description: 'Connect another device',
      icon: <QrCode className="w-4 h-4" />,
      keywords: ['sync', 'pair', 'device', 'connect', 'qr'],
      action: () => {
        // TODO: Start pairing
        console.log('Pair device');
        onClose();
      },
      section: 'Sync',
    },
    
    // Export Commands
    {
      id: 'export-obsidian',
      title: 'Export to Obsidian',
      description: 'Export wiki as Obsidian vault',
      icon: <FileDown className="w-4 h-4" />,
      keywords: ['export', 'obsidian', 'markdown', 'backup'],
      action: () => {
        // TODO: Export to Obsidian
        console.log('Export to Obsidian');
        onClose();
      },
      section: 'Export',
    },
    {
      id: 'export-markdown',
      title: 'Export as Markdown',
      description: 'Export all content as markdown files',
      icon: <FileDown className="w-4 h-4" />,
      keywords: ['export', 'markdown', 'files', 'backup'],
      action: () => {
        // TODO: Export markdown
        console.log('Export markdown');
        onClose();
      },
      section: 'Export',
    },
    
    // Settings Commands
    {
      id: 'settings',
      title: 'Open Settings',
      description: 'Configure Truffle preferences',
      icon: <Settings className="w-4 h-4" />,
      shortcut: '⌘,',
      keywords: ['settings', 'preferences', 'config', 'options'],
      action: () => {
        // TODO: Open settings
        console.log('Open settings');
        onClose();
      },
      section: 'Settings',
    },
    {
      id: 'toggle-theme',
      title: 'Toggle Theme',
      description: 'Switch between light and dark mode',
      icon: <Moon className="w-4 h-4" />,
      keywords: ['theme', 'dark', 'light', 'mode', 'appearance'],
      action: () => {
        // TODO: Toggle theme
        console.log('Toggle theme');
        onClose();
      },
      section: 'Settings',
    },
    
    // Help Commands
    {
      id: 'keyboard-shortcuts',
      title: 'Keyboard Shortcuts',
      description: 'View all available shortcuts',
      icon: <Keyboard className="w-4 h-4" />,
      keywords: ['help', 'keyboard', 'shortcuts', 'hotkeys'],
      action: () => {
        // TODO: Show keyboard shortcuts
        console.log('Keyboard shortcuts');
        onClose();
      },
      section: 'Help',
    },
    {
      id: 'documentation',
      title: 'Documentation',
      description: 'Open Truffle documentation',
      icon: <HelpCircle className="w-4 h-4" />,
      keywords: ['help', 'docs', 'documentation', 'guide'],
      action: () => {
        // TODO: Open docs
        console.log('Open documentation');
        onClose();
      },
      section: 'Help',
    },
  ], [artifacts, setViewMode, setIsCompiling, openSearch, onClose]);

  // Filter commands based on query
  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands;
    
    const searchTerms = query.toLowerCase().split(' ');
    
    return commands.filter(cmd => {
      const searchText = `${cmd.title} ${cmd.description} ${cmd.keywords.join(' ')}`.toLowerCase();
      return searchTerms.every(term => searchText.includes(term));
    });
  }, [commands, query]);

  // Group commands by section
  const groupedCommands = useMemo(() => {
    const groups: Record<string, CommandItem[]> = {};
    
    filteredCommands.forEach(cmd => {
      if (!groups[cmd.section]) {
        groups[cmd.section] = [];
      }
      groups[cmd.section].push(cmd);
    });
    
    return groups;
  }, [filteredCommands]);

  // Flat list for keyboard navigation
  const flatCommands = useMemo(() => {
    return Object.values(groupedCommands).flat();
  }, [groupedCommands]);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Reset selection when query changes
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, flatCommands.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (flatCommands[selectedIndex]) {
          flatCommands[selectedIndex].action();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }, [flatCommands, selectedIndex, onClose]);

  // Get selected command ID
  const selectedCommandId = flatCommands[selectedIndex]?.id;

  return (
    <div 
      className={`
        fixed inset-0 z-command-palette
        bg-black/60 backdrop-blur-sm
        flex items-start justify-center pt-[15vh]
        ${className}
      `}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div 
        className="w-full max-w-2xl mx-4 bg-surface-elevated rounded-xl shadow-2xl border border-border overflow-hidden command-palette-enter"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Command className="w-5 h-5 text-text-muted" />
          
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search..."
            className="flex-1 bg-transparent text-lg text-text-primary placeholder:text-text-muted focus:outline-none"
          />
          
          {query && (
            <button
              onClick={() => {
                setQuery('');
                inputRef.current?.focus();
              }}
              className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-surface-hover transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          )}
          
          <kbd className="hidden sm:flex items-center gap-1 px-2 py-1 bg-background rounded text-xs text-text-muted">
            <span>ESC</span>
          </kbd>
        </div>

        {/* Commands */}
        <div className="max-h-[50vh] overflow-auto py-2">
          {flatCommands.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-text-muted">
              <Search className="w-10 h-10 mb-3 opacity-50" />
              <p className="text-sm">No commands found</p>
              <p className="text-xs mt-1">Try a different search term</p>
            </div>
          ) : (
            Object.entries(groupedCommands).map(([section, items]) => (
              <div key={section}>
                {/* Section Header */}
                <div className="px-4 py-1.5 text-xs font-medium text-text-muted uppercase tracking-wide">
                  {section}
                </div>
                
                {/* Section Items */}
                <div className="px-2">
                  {items.map((command) => (
                    <button
                      key={command.id}
                      onClick={command.action}
                      onMouseEnter={() => {
                        const index = flatCommands.findIndex(c => c.id === command.id);
                        setSelectedIndex(index);
                      }}
                      className={`
                        w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left
                        transition-all duration-150
                        ${selectedCommandId === command.id 
                          ? 'bg-primary/10' 
                          : 'hover:bg-surface-hover'
                        }
                      `}
                    >
                      <span className={`
                        ${selectedCommandId === command.id 
                          ? 'text-primary' 
                          : 'text-text-muted'
                        }
                      `}>
                        {command.icon}
                      </span>
                      
                      <div className="flex-1 min-w-0">
                        <p className={`
                          text-sm font-medium
                          ${selectedCommandId === command.id 
                            ? 'text-primary' 
                            : 'text-text-primary'
                          }
                        `}>
                          {command.title}
                        </p>
                        {command.description && (
                          <p className="text-xs text-text-muted mt-0.5">
                            {command.description}
                          </p>
                        )}
                      </div>
                      
                      {command.shortcut && (
                        <kbd className="px-2 py-0.5 bg-background rounded text-xs text-text-muted">
                          {command.shortcut}
                        </kbd>
                      )}
                    </button>
                  ))}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-border bg-surface/50 text-xs text-text-muted">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1">
              <span className="px-1 bg-background rounded">↑↓</span> to navigate
            </span>
            <span className="flex items-center gap-1">
              <span className="px-1 bg-background rounded">↵</span> to select
            </span>
          </div>
          
          <div className="flex items-center gap-2">
            <span>{flatCommands.length} commands</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default CommandPalette;
