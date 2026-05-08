// Export all components

// Layout
export { AppLayout } from './Layout/AppLayout';
export { Header } from './Layout/Header';
export { StatusBar } from './Layout/StatusBar';

// Raw Panel
export { RawPanel } from './Raw/RawPanel';
export { ThumbnailGrid } from './Raw/ThumbnailGrid';
export { DragDropZone } from './Raw/DragDropZone';

// Compilation Panel
export { CompilationPanel } from './Compilation/CompilationPanel';
export { BeforeAfterDiff } from './Compilation/BeforeAfterDiff';
export { GemmaLogs } from './Compilation/GemmaLogs';

// Wiki Panel
export { WikiPanel } from './Wiki/WikiPanel';
export { TreeView } from './Wiki/TreeView';
export { WikiNode } from './Wiki/WikiNode';
export { Backlinks } from './Wiki/Backlinks';

// Editor
export { MilkdownEditor, EditorToolbar } from './Editor/MilkdownEditor';
export { WikiLinkPlugin, wikiLinkStyles } from './Editor/WikiLinkPlugin';

// Search
export { SearchBar } from './Search/SearchBar';
export { ResultsList } from './Search/ResultsList';

// Command Palette
export { CommandPalette } from './CommandPalette/CommandPalette';

// Entity Management
export {
  EntityList,
  EntityDetail,
  EntityForm,
  EntityCard,
  EntityTypeBadge,
  EntityTypeIcon,
  getEntityTypeColor,
} from './Entity';
export type {
  EntityListProps,
  EntityDetailProps,
  EntityFormProps,
  EntityCardProps,
  EntityTypeBadgeProps,
} from './Entity';
