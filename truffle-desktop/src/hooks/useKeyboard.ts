// Custom hook for keyboard shortcuts

import { useEffect, useCallback, useRef } from 'react';

interface KeyBinding {
  key: string;
  modifiers?: {
    ctrl?: boolean;
    meta?: boolean;
    shift?: boolean;
    alt?: boolean;
  };
  handler: (event: KeyboardEvent) => void;
  preventDefault?: boolean;
}

export function useKeyboard(bindings: KeyBinding[]) {
  const bindingsRef = useRef(bindings);
  bindingsRef.current = bindings;

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    for (const binding of bindingsRef.current) {
      const { key, modifiers = {}, handler, preventDefault = true } = binding;
      
      // Check if key matches
      if (event.key.toLowerCase() !== key.toLowerCase()) {
        continue;
      }
      
      // Check modifiers
      const ctrlMatch = modifiers.ctrl === undefined || event.ctrlKey === modifiers.ctrl;
      const metaMatch = modifiers.meta === undefined || event.metaKey === modifiers.meta;
      const shiftMatch = modifiers.shift === undefined || event.shiftKey === modifiers.shift;
      const altMatch = modifiers.alt === undefined || event.altKey === modifiers.alt;
      
      if (ctrlMatch && metaMatch && shiftMatch && altMatch) {
        if (preventDefault) {
          event.preventDefault();
        }
        handler(event);
        break;
      }
    }
  }, []);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

// Hook for single shortcut
export function useShortcut(
  key: string,
  handler: () => void,
  modifiers?: {
    ctrl?: boolean;
    meta?: boolean;
    shift?: boolean;
    alt?: boolean;
  }
) {
  useKeyboard([
    {
      key,
      modifiers,
      handler,
    },
  ]);
}

// Hook for escape key
export function useEscape(handler: () => void) {
  useShortcut('Escape', handler);
}

// Hook for enter key
export function useEnter(handler: () => void) {
  useShortcut('Enter', handler);
}
