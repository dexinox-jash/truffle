// SPEC v2.0 SECTION 5.2.2: Header Component
// Navigation bar for Truffle Desktop

import React from 'react';
import { 
  Image, 
  BookOpen, 
  Columns, 
  Search, 
  Command, 
  Settings,
  RefreshCw,
  Cloud,
  CloudOff,
  Zap
} from 'lucide-react';
import { useAppStore, ViewMode } from '../../store/appStore';
import { useSync } from '../../store/syncStore';

interface HeaderProps {
  className?: string;
}

export const Header: React.FC<HeaderProps> = ({ className = '' }) => {
  const { 
    viewMode, 
    setViewMode, 
    isCompiling,
    pendingChanges,
    openCommandPalette,
    openSearch,
  } = useAppStore();
  
  const { isConnected, syncHealth } = useSync();

  const viewButtons: { mode: ViewMode; icon: React.ReactNode; label: string }[] = [
    { mode: 'raw', icon: <Image className="w-4 h-4" />, label: 'Raw' },
    { mode: 'split', icon: <Columns className="w-4 h-4" />, label: 'Split' },
    { mode: 'wiki', icon: <BookOpen className="w-4 h-4" />, label: 'Wiki' },
  ];

  const getSyncIcon = () => {
    if (!isConnected) return <CloudOff className="w-4 h-4 text-text-muted" />;
    if (syncHealth === 'error') return <CloudOff className="w-4 h-4 text-error" />;
    if (pendingChanges > 0) return <RefreshCw className="w-4 h-4 text-warning animate-spin" />;
    return <Cloud className="w-4 h-4 text-secondary" />;
  };

  return (
    <header 
      className={`
        h-12 flex items-center justify-between px-4 
        bg-surface border-b border-border
        ${className}
      `}
    >
      {/* Left: Logo & View Switcher */}
      <div className="flex items-center gap-6">
        {/* Logo */}
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded bg-primary flex items-center justify-center">
            <Zap className="w-4 h-4 text-background" />
          </div>
          <span className="font-semibold text-lg tracking-tight">Truffle</span>
        </div>
        
        {/* View Mode Switcher */}
        <div className="flex items-center gap-1 bg-background rounded-lg p-1">
          {viewButtons.map(({ mode, icon, label }) => (
            <button
              key={mode}
              onClick={() => setViewMode(mode)}
              className={`
                flex items-center gap-2 px-3 py-1.5 rounded-md text-sm font-medium
                transition-all duration-200
                ${viewMode === mode 
                  ? 'bg-surface-elevated text-text-primary shadow-sm' 
                  : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
                }
              `}
            >
              {icon}
              <span className="hidden sm:inline">{label}</span>
            </button>
          ))}
        </div>
      </div>
      
      {/* Right: Actions */}
      <div className="flex items-center gap-2">
        {/* Search Button */}
        <button
          onClick={openSearch}
          className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
        >
          <Search className="w-4 h-4" />
          <span className="hidden md:inline">Search</span>
          <kbd className="hidden lg:inline-flex items-center gap-1 px-1.5 py-0.5 bg-background rounded text-xs text-text-muted">
            <span className="text-[10px]">⌘</span>⇧F
          </kbd>
        </button>
        
        {/* Command Palette Button */}
        <button
          onClick={openCommandPalette}
          className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
        >
          <Command className="w-4 h-4" />
          <span className="hidden md:inline">Commands</span>
          <kbd className="hidden lg:inline-flex items-center gap-1 px-1.5 py-0.5 bg-background rounded text-xs text-text-muted">
            <span className="text-[10px]">⌘</span>K
          </kbd>
        </button>
        
        {/* Divider */}
        <div className="w-px h-6 bg-border mx-1" />
        
        {/* Sync Status */}
        <button
          className="flex items-center gap-2 px-2 py-1.5 rounded-md hover:bg-surface-hover transition-colors"
          title={isConnected ? 'Sync active' : 'Sync offline'}
        >
          {getSyncIcon()}
          {pendingChanges > 0 && (
            <span className="text-xs text-warning">{pendingChanges}</span>
          )}
        </button>
        
        {/* Settings */}
        <button
          className="p-2 rounded-md text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
          title="Settings"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};

export default Header;
