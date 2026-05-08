import React, { createContext, useContext, useState } from 'react';
import { cn } from '../../utils/cn';

interface TabsContextValue {
  value: string;
  onChange: (value: string) => void;
}

const TabsContext = createContext<TabsContextValue | null>(null);

const useTabs = () => {
  const context = useContext(TabsContext);
  if (!context) {
    throw new Error('Tabs components must be used within a Tabs provider');
  }
  return context;
};

// ==================== Tabs Container ====================

interface TabsProps {
  value: string;
  onChange: (value: string) => void;
  children: React.ReactNode;
  className?: string;
}

export const Tabs: React.FC<TabsProps> = ({ value, onChange, children, className }) => {
  return (
    <TabsContext.Provider value={{ value, onChange }}>
      <div className={cn('flex flex-col h-full', className)}>{children}</div>
    </TabsContext.Provider>
  );
};

// ==================== Tab List ====================

interface TabListProps {
  children: React.ReactNode;
  className?: string;
}

export const TabList: React.FC<TabListProps> = ({ children, className }) => {
  return (
    <div className={cn('flex items-center border-b border-darkroom-gray-700', className)}>
      {children}
    </div>
  );
};

// ==================== Tab ====================

interface TabProps {
  value: string;
  children: React.ReactNode;
  className?: string;
}

export const Tab: React.FC<TabProps> = ({ value, children, className }) => {
  const { value: selectedValue, onChange } = useTabs();
  const isActive = selectedValue === value;

  return (
    <button
      onClick={() => onChange(value)}
      className={cn(
        'px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px',
        isActive
          ? 'text-accent border-accent'
          : 'text-darkroom-text-secondary border-transparent hover:text-darkroom-text-primary',
        className
      )}
    >
      {children}
    </button>
  );
};

// ==================== Tab Panels ====================

interface TabPanelsProps {
  children: React.ReactNode;
  className?: string;
}

export const TabPanels: React.FC<TabPanelsProps> = ({ children, className }) => {
  return <div className={cn('flex-1', className)}>{children}</div>;
};

// ==================== Tab Panel ====================

interface TabPanelProps {
  value: string;
  children: React.ReactNode;
  className?: string;
}

export const TabPanel: React.FC<TabPanelProps> = ({ value, children, className }) => {
  const { value: selectedValue } = useTabs();

  if (value !== selectedValue) {
    return null;
  }

  return <div className={cn('h-full', className)}>{children}</div>;
};
