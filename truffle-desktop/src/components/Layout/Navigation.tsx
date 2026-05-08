import React, { useEffect, useCallback } from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import {
  Image,
  FileText,
  Database,
  Share2,
  Calendar,
  ShieldAlert,
  Settings,
  Search,
  Command,
} from 'lucide-react';
import { useKeyboardShortcut } from '../../hooks/useKeyboardShortcut';
import { useNavigationStore } from '../../store/navigationStore';

interface NavItem {
  path: string;
  label: string;
  icon: React.ElementType;
  shortcut?: string;
}

const navItems: NavItem[] = [
  { path: '/raw', label: 'Raw', icon: Image, shortcut: 'mod+r' },
  { path: '/wiki', label: 'Wiki', icon: FileText, shortcut: 'mod+w' },
  { path: '/entities', label: 'Entities', icon: Database, shortcut: 'mod+e' },
  { path: '/graph', label: 'Graph', icon: Share2, shortcut: 'mod+g' },
  { path: '/meetings', label: 'Meetings', icon: Calendar, shortcut: 'mod+m' },
  { path: '/validation', label: 'Validation', icon: ShieldAlert, shortcut: 'mod+shift+v' },
];

const bottomNavItems: NavItem[] = [
  { path: '/search', label: 'Search', icon: Search },
  { path: '/settings', label: 'Settings', icon: Settings },
];

export const Navigation: React.FC = () => {
  const location = useLocation();
  const { isCollapsed, setCollapsed } = useNavigationStore();

  // Keyboard shortcuts for navigation
  useKeyboardShortcut('mod+e', () => {
    window.location.href = '/entities';
  });

  useKeyboardShortcut('mod+g', () => {
    window.location.href = '/graph';
  });

  useKeyboardShortcut('mod+m', () => {
    window.location.href = '/meetings';
  });

  useKeyboardShortcut('mod+shift+v', () => {
    window.location.href = '/validation';
  });

  return (
    <nav
      className={`h-screen bg-darkroom-card border-r border-darkroom-gray-700 flex flex-col transition-all duration-200 ${
        isCollapsed ? 'w-16' : 'w-60'
      }`}
    >
      {/* Logo */}
      <div className="h-16 flex items-center px-4 border-b border-darkroom-gray-700">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-accent rounded-lg flex items-center justify-center">
            <Command className="w-5 h-5 text-white" />
          </div>
          {!isCollapsed && (
            <span className="font-bold text-darkroom-text-primary text-lg">
              Truffle
            </span>
          )}
        </div>
      </div>

      {/* Main navigation */}
      <div className="flex-1 py-4 space-y-1 overflow-y-auto">
        {navItems.map((item) => (
          <NavItemComponent
            key={item.path}
            item={item}
            isCollapsed={isCollapsed}
            isActive={location.pathname.startsWith(item.path)}
          />
        ))}
      </div>

      {/* Bottom navigation */}
      <div className="py-4 border-t border-darkroom-gray-700 space-y-1">
        {bottomNavItems.map((item) => (
          <NavItemComponent
            key={item.path}
            item={item}
            isCollapsed={isCollapsed}
            isActive={location.pathname === item.path}
          />
        ))}

        {/* Collapse toggle */}
        <button
          onClick={() => setCollapsed(!isCollapsed)}
          className="w-full px-4 py-2 flex items-center gap-3 text-darkroom-text-secondary hover:text-darkroom-text-primary hover:bg-darkroom-gray-700/50 transition-colors"
        >
          <ChevronIcon collapsed={isCollapsed} />
          {!isCollapsed && <span className="text-sm">Collapse</span>}
        </button>
      </div>
    </nav>
  );
};

// Subcomponents

interface NavItemComponentProps {
  item: NavItem;
  isCollapsed: boolean;
  isActive: boolean;
}

const NavItemComponent: React.FC<NavItemComponentProps> = ({
  item,
  isCollapsed,
  isActive,
}) => {
  const Icon = item.icon;

  return (
    <NavLink
      to={item.path}
      className={({ isActive }) =>
        `w-full px-4 py-2 flex items-center gap-3 transition-colors ${
          isActive
            ? 'bg-accent/10 text-accent border-r-2 border-accent'
            : 'text-darkroom-text-secondary hover:text-darkroom-text-primary hover:bg-darkroom-gray-700/50'
        }`
      }
    >
      <Icon className={`w-5 h-5 ${isActive ? 'text-accent' : ''}`} />
      {!isCollapsed && (
        <div className="flex items-center justify-between flex-1">
          <span className="text-sm font-medium">{item.label}</span>
          {item.shortcut && (
            <ShortcutBadge shortcut={item.shortcut} />
          )}
        </div>
      )}
    </NavLink>
  );
};

const ShortcutBadge: React.FC<{ shortcut: string }> = ({ shortcut }) => {
  const display = shortcut
    .replace('mod+', '⌘')
    .replace('shift+', '⇧')
    .replace('alt+', '⌥')
    .toUpperCase();

  return (
    <kbd className="hidden lg:inline-block px-1.5 py-0.5 text-[10px] bg-darkroom-gray-700 text-darkroom-text-tertiary rounded">
      {display}
    </kbd>
  );
};

const ChevronIcon: React.FC<{ collapsed: boolean }> = ({ collapsed }) => (
  <svg
    className={`w-5 h-5 transition-transform ${collapsed ? 'rotate-180' : ''}`}
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
    />
  </svg>
);

export default Navigation;
