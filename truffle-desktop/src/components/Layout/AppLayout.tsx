// SPEC v2.0 SECTION 5.2.2: Split Soul Interface
// Main layout component for Truffle Desktop

import React, { useEffect, useCallback } from 'react';
import { useAppStore, ViewMode } from '../../store/appStore';
import { Header } from './Header';
import { Navigation } from './Navigation';
import { StatusBar } from './StatusBar';
import { RawPanel } from '../Raw/RawPanel';
import { CompilationPanel } from '../Compilation/CompilationPanel';
import { WikiPanel } from '../Wiki/WikiPanel';
import { CommandPalette } from '../CommandPalette/CommandPalette';
import { SearchBar } from '../Search/SearchBar';

interface AppLayoutProps {
  children?: React.ReactNode;
}

export const AppLayout: React.FC<AppLayoutProps> = ({ children }) => {
  const {
    viewMode,
    panelFocus,
    isCommandPaletteOpen,
    isSearchOpen,
    setViewMode,
    setPanelFocus,
    openCommandPalette,
    closeCommandPalette,
    openSearch,
    closeSearch,
  } = useAppStore();

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + K - Command Palette
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        openCommandPalette();
      }
      
      // Cmd/Ctrl + Shift + F - Search
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'F') {
        e.preventDefault();
        openSearch();
      }
      
      // Cmd/Ctrl + 1 - Raw view
      if ((e.metaKey || e.ctrlKey) && e.key === '1') {
        e.preventDefault();
        setViewMode('raw');
      }
      
      // Cmd/Ctrl + 2 - Split view
      if ((e.metaKey || e.ctrlKey) && e.key === '2') {
        e.preventDefault();
        setViewMode('split');
      }
      
      // Cmd/Ctrl + 3 - Wiki view
      if ((e.metaKey || e.ctrlKey) && e.key === '3') {
        e.preventDefault();
        setViewMode('wiki');
      }
      
      // Escape - Close modals
      if (e.key === 'Escape') {
        closeCommandPalette();
        closeSearch();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [openCommandPalette, closeCommandPalette, openSearch, closeSearch, setViewMode]);

  // Handle panel focus change
  const handlePanelClick = useCallback((focus: typeof panelFocus) => {
    setPanelFocus(focus);
  }, [setPanelFocus]);

  // Render panels based on view mode (when no children provided)
  const renderPanels = () => {
    switch (viewMode) {
      case 'raw':
        return (
          <div className="flex-1 h-full overflow-hidden">
            <RawPanel className="w-full h-full" />
          </div>
        );
        
      case 'wiki':
        return (
          <div className="flex-1 h-full overflow-hidden">
            <WikiPanel className="w-full h-full" />
          </div>
        );
        
      case 'split':
      default:
        return (
          <div className="flex-1 flex h-full overflow-hidden">
            {/* Left Panel - Raw Waterfall */}
            <div 
              className={`
                flex-shrink-0 h-full overflow-hidden border-r border-border
                transition-all duration-200 ease-out
                ${panelFocus === 'left' ? 'w-[35%]' : 'w-[30%]'}
              `}
              onClick={() => handlePanelClick('left')}
            >
              <RawPanel className="w-full h-full" />
            </div>
            
            {/* Center Panel - Compilation Preview */}
            <div 
              className={`
                flex-shrink-0 h-full overflow-hidden border-r border-border
                transition-all duration-200 ease-out
                ${panelFocus === 'center' ? 'w-[35%]' : 'w-[30%]'}
              `}
              onClick={() => handlePanelClick('center')}
            >
              <CompilationPanel className="w-full h-full" />
            </div>
            
            {/* Right Panel - Wiki Navigator */}
            <div 
              className={`
                flex-1 h-full overflow-hidden
                transition-all duration-200 ease-out
              `}
              onClick={() => handlePanelClick('right')}
            >
              <WikiPanel className="w-full h-full" />
            </div>
          </div>
        );
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-background text-text-primary overflow-hidden">
      {/* Header */}
      <Header />
      
      {/* Main Content Area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Navigation Sidebar - shown when children provided (routed mode) */}
        {children && <Navigation />}
        
        {/* Content */}
        <main className="flex-1 flex overflow-hidden">
          {children || renderPanels()}
        </main>
      </div>
      
      {/* Status Bar */}
      <StatusBar />
      
      {/* Modals */}
      {isCommandPaletteOpen && (
        <CommandPalette onClose={closeCommandPalette} />
      )}
      
      {isSearchOpen && (
        <SearchBar onClose={closeSearch} />
      )}
    </div>
  );
};

export default AppLayout;
