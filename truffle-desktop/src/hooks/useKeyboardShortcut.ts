import { useEffect, useCallback } from 'react';

type ModifierKey = 'mod' | 'ctrl' | 'alt' | 'shift' | 'meta';
type ShortcutKey = string;

interface ShortcutOptions {
  preventDefault?: boolean;
  enabled?: boolean;
}

/**
 * Parse a shortcut string like "mod+k" or "ctrl+shift+f" into components
 */
const parseShortcut = (shortcut: string): { modifiers: Set<ModifierKey>; key: string } => {
  const parts = shortcut.toLowerCase().split('+');
  const modifiers = new Set<ModifierKey>();
  let key = '';

  for (const part of parts) {
    if (['mod', 'ctrl', 'alt', 'shift', 'meta'].includes(part)) {
      modifiers.add(part as ModifierKey);
    } else {
      key = part;
    }
  }

  return { modifiers, key };
};

/**
 * Check if the keyboard event matches the shortcut
 */
const matchesShortcut = (event: KeyboardEvent, modifiers: Set<ModifierKey>, key: string): boolean => {
  // Check if key matches (handle special cases)
  const eventKey = event.key.toLowerCase();
  if (eventKey !== key && event.code.toLowerCase() !== `key${key.toUpperCase()}`.toLowerCase()) {
    return false;
  }

  // Check modifiers
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  
  // Handle 'mod' key (Cmd on Mac, Ctrl on Windows/Linux)
  if (modifiers.has('mod')) {
    if (isMac) {
      if (!event.metaKey) return false;
    } else {
      if (!event.ctrlKey) return false;
    }
  }

  // Check other modifiers
  if (modifiers.has('ctrl') && !event.ctrlKey) return false;
  if (modifiers.has('meta') && !event.metaKey) return false;
  if (modifiers.has('alt') && !event.altKey) return false;
  if (modifiers.has('shift') && !event.shiftKey) return false;

  // Ensure no extra modifiers are pressed
  const expectedMod = modifiers.has('mod');
  const expectedCtrl = modifiers.has('ctrl') || (!isMac && expectedMod);
  const expectedMeta = modifiers.has('meta') || (isMac && expectedMod);
  const expectedAlt = modifiers.has('alt');
  const expectedShift = modifiers.has('shift');

  if (event.ctrlKey !== expectedCtrl) return false;
  if (event.metaKey !== expectedMeta) return false;
  if (event.altKey !== expectedAlt) return false;
  if (event.shiftKey !== expectedShift) return false;

  return true;
};

/**
 * Hook for registering keyboard shortcuts
 * @param shortcut - The keyboard shortcut (e.g., "mod+k", "ctrl+shift+f")
 * @param callback - Function to call when shortcut is triggered
 * @param options - Options for the shortcut
 */
export const useKeyboardShortcut = (
  shortcut: ShortcutKey,
  callback: () => void,
  options: ShortcutOptions = {}
): void => {
  const { preventDefault = true, enabled = true } = options;

  const { modifiers, key } = parseShortcut(shortcut);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      if (matchesShortcut(event, modifiers, key)) {
        if (preventDefault) {
          event.preventDefault();
        }
        callback();
      }
    },
    [enabled, preventDefault, callback, modifiers, key]
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
};

export default useKeyboardShortcut;
