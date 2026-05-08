// SPEC v2.0 SECTION 5.2.1: Custom WikiLink Plugin for Milkdown
// Enables [[WikiLink]] syntax for internal linking

import { $prose, $inputRule } from '@milkdown/utils';
import { NodeType } from 'prosemirror-model';
import { Plugin, PluginKey } from 'prosemirror-state';
import { Decoration, DecorationSet } from 'prosemirror-view';
import { InputRule } from 'prosemirror-inputrules';

// WikiLink node spec
export const wikiLinkSchema = {
  wikiLink: {
    group: 'inline',
    inline: true,
    atom: true,
    attrs: {
      target: { default: '' },
      display: { default: '' },
    },
    parseDOM: [
      {
        tag: 'span[data-wiki-link]',
        getAttrs: (dom: HTMLElement) => ({
          target: dom.getAttribute('data-target') || '',
          display: dom.textContent || '',
        }),
      },
    ],
    toDOM: (node: any) => [
      'span',
      {
        'data-wiki-link': 'true',
        'data-target': node.attrs.target,
        class: 'wiki-link',
      },
      node.attrs.display || node.attrs.target,
    ],
  },
};

// Plugin key for wiki link decorations
const wikiLinkPluginKey = new PluginKey('wikiLink');

// WikiLink decoration plugin
const wikiLinkDecorationPlugin = new Plugin({
  key: wikiLinkPluginKey,
  state: {
    init() {
      return DecorationSet.empty;
    },
    apply(tr, set) {
      // Update decorations on transaction
      set = set.map(tr.mapping, tr.doc);
      
      // Find wiki link patterns in text nodes
      const decorations: Decoration[] = [];
      const wikiLinkRegex = /\[\[([^\]]+)\]\]/g;
      
      tr.doc.descendants((node, pos) => {
        if (node.isText) {
          const text = node.text || '';
          let match;
          
          while ((match = wikiLinkRegex.exec(text)) !== null) {
            const from = pos + match.index;
            const to = from + match[0].length;
            const target = match[1];
            
            // Create decoration for wiki link
            decorations.push(
              Decoration.inline(from, to, {
                class: 'wiki-link-highlight',
                'data-wiki-target': target,
              })
            );
          }
        }
      });
      
      return DecorationSet.create(tr.doc, decorations);
    },
  },
  props: {
    decorations(state) {
      return this.getState(state);
    },
    handleClickOn(view, pos, node, nodePos, event) {
      const target = (event.target as HTMLElement).getAttribute('data-wiki-target');
      if (target) {
        // Emit wiki link click event
        const customEvent = new CustomEvent('wikilink:click', {
          detail: { target, view },
        });
        document.dispatchEvent(customEvent);
        return true;
      }
      return false;
    },
  },
});

// Input rule for creating wiki links
const wikiLinkInputRule = $inputRule((ctx) => {
  return new InputRule(
    /\[\[([^\]]+)\]\]$/,
    (state, match, start, end) => {
      const target = match[1];
      const display = target;
      
      // Create wiki link node
      const nodeType = state.schema.nodes.wikiLink;
      if (!nodeType) return null;
      
      const node = nodeType.create({ target, display });
      return state.tr.replaceWith(start, end, node);
    }
  );
});

// Main WikiLink plugin
export const WikiLinkPlugin = $prose(() => wikiLinkDecorationPlugin);

// WikiLink suggestion plugin (for autocomplete)
export const wikiLinkSuggestionPlugin = (getSuggestions: (query: string) => string[]) => {
  return new Plugin({
    key: new PluginKey('wikiLinkSuggestion'),
    props: {
      handleTextInput(view, from, to, text) {
        // Check if user is typing a wiki link
        const state = view.state;
        const $from = state.doc.resolve(from);
        const textBefore = $from.parent.textContent.slice(0, $from.parentOffset);
        
        const match = textBefore.match(/\[\[([^\]]*)$/);
        if (match) {
          const query = match[1];
          const suggestions = getSuggestions(query);
          
          // Emit suggestion event
          const event = new CustomEvent('wikilink:suggest', {
            detail: { query, suggestions, view, from, to },
          });
          document.dispatchEvent(event);
        }
        
        return false;
      },
    },
  });
};

// CSS styles for wiki links
export const wikiLinkStyles = `
  .wiki-link-highlight {
    color: #FFB800;
    background: rgba(255, 184, 0, 0.1);
    border-radius: 3px;
    padding: 0 4px;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  
  .wiki-link-highlight:hover {
    background: rgba(255, 184, 0, 0.2);
    text-decoration: underline;
  }
  
  .wiki-link {
    color: #FFB800;
    text-decoration: none;
    cursor: pointer;
  }
  
  .wiki-link:hover {
    text-decoration: underline;
  }
  
  .wiki-link-suggestions {
    position: absolute;
    background: #141414;
    border: 1px solid #262626;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    max-height: 200px;
    overflow-y: auto;
  }
  
  .wiki-link-suggestion {
    padding: 8px 12px;
    cursor: pointer;
    color: #E5E5E5;
    font-size: 14px;
  }
  
  .wiki-link-suggestion:hover,
  .wiki-link-suggestion.selected {
    background: rgba(255, 184, 0, 0.1);
    color: #FFB800;
  }
`;

// Utility function to extract wiki links from content
export function extractWikiLinks(content: string): string[] {
  const links: string[] = [];
  const regex = /\[\[([^\]|]+)(?:\|[^\]]+)?\]\]/g;
  let match;
  
  while ((match = regex.exec(content)) !== null) {
    links.push(match[1]);
  }
  
  return [...new Set(links)]; // Remove duplicates
}

// Utility function to create wiki link
export function createWikiLink(target: string, display?: string): string {
  if (display && display !== target) {
    return `[[${target}|${display}]]`;
  }
  return `[[${target}]]`;
}

// Utility function to resolve wiki link
export function resolveWikiLink(link: string, availableNodes: string[]): string | null {
  // Exact match
  if (availableNodes.includes(link)) {
    return link;
  }
  
  // Case-insensitive match
  const lowerLink = link.toLowerCase();
  const match = availableNodes.find(n => n.toLowerCase() === lowerLink);
  if (match) {
    return match;
  }
  
  // Fuzzy match (contains)
  const fuzzyMatch = availableNodes.find(n => 
    n.toLowerCase().includes(lowerLink) || 
    lowerLink.includes(n.toLowerCase())
  );
  
  return fuzzyMatch || null;
}

export default WikiLinkPlugin;
