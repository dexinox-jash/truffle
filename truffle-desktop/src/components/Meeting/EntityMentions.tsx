import { useMemo, useCallback, useState } from 'react';
import { MapPin, ArrowUpRight } from 'lucide-react';
import type { Entity } from '../../types/knowledge-graph';
import { EntityType } from '../../types/knowledge-graph';

export interface EntityMentionsProps {
  transcript: string;
  entities: Entity[];
  onMentionClick?: (entity: Entity) => void;
}

interface Mention {
  entity: Entity;
  startIndex: number;
  endIndex: number;
  text: string;
}

// Entity type colors for highlighting
const entityHighlightColors: Record<EntityType, string> = {
  [EntityType.Person]: 'bg-blue-500/20 text-blue-300 border-blue-500/30 hover:bg-blue-500/30',
  [EntityType.Organization]: 'bg-green-500/20 text-green-300 border-green-500/30 hover:bg-green-500/30',
  [EntityType.Location]: 'bg-orange-500/20 text-orange-300 border-orange-500/30 hover:bg-orange-500/30',
  [EntityType.Event]: 'bg-purple-500/20 text-purple-300 border-purple-500/30 hover:bg-purple-500/30',
  [EntityType.Product]: 'bg-pink-500/20 text-pink-300 border-pink-500/30 hover:bg-pink-500/30',
  [EntityType.Concept]: 'bg-gray-500/20 text-gray-300 border-gray-500/30 hover:bg-gray-500/30',
  [EntityType.Decision]: 'bg-yellow-500/20 text-yellow-300 border-yellow-500/30 hover:bg-yellow-500/30',
  [EntityType.ActionItem]: 'bg-red-500/20 text-red-300 border-red-500/30 hover:bg-red-500/30',
};

function findMentions(transcript: string, entities: Entity[]): Mention[] {
  const mentions: Mention[] = [];
  
  entities.forEach((entity) => {
    // Search for entity name and aliases in transcript
    const searchTerms = [entity.name, ...(entity.metadata?.aliases || [])];
    
    searchTerms.forEach((term) => {
      if (!term || term.length < 2) return;
      
      const regex = new RegExp(`\\b${escapeRegex(term)}\\b`, 'gi');
      let match;
      
      while ((match = regex.exec(transcript)) !== null) {
        mentions.push({
          entity,
          startIndex: match.index,
          endIndex: match.index + match[0].length,
          text: match[0],
        });
      }
    });
  });
  
  // Sort by start index and remove overlapping mentions (prefer longer matches)
  mentions.sort((a, b) => a.startIndex - b.startIndex);
  
  const filtered: Mention[] = [];
  let lastEnd = -1;
  
  for (const mention of mentions) {
    if (mention.startIndex >= lastEnd) {
      filtered.push(mention);
      lastEnd = mention.endIndex;
    }
  }
  
  return filtered;
}

function escapeRegex(string: string): string {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function EntityTooltip({ 
  entity, 
  isVisible, 
  position 
}: { 
  entity: Entity; 
  isVisible: boolean;
  position: { x: number; y: number };
}) {
  if (!isVisible) return null;

  return (
    <div
      className="
        fixed z-50 p-3 bg-gray-800 border border-gray-700 rounded-lg shadow-xl
        max-w-xs pointer-events-none
      "
      style={{ 
        left: position.x, 
        top: position.y - 10,
        transform: 'translate(-50%, -100%)'
      }}
    >
      <div className="flex items-start gap-2">
        <div className={`
          w-2 h-2 rounded-full mt-1.5 flex-shrink-0
          ${entity.entityType === EntityType.Person ? 'bg-blue-500' : ''}
          ${entity.entityType === EntityType.Organization ? 'bg-green-500' : ''}
          ${entity.entityType === EntityType.Location ? 'bg-orange-500' : ''}
          ${entity.entityType === EntityType.Decision ? 'bg-yellow-500' : ''}
          ${entity.entityType === EntityType.ActionItem ? 'bg-red-500' : ''}
        `} />
        <div className="flex-1 min-w-0">
          <p className="font-medium text-gray-200 text-sm truncate">{entity.name}</p>
          <p className="text-xs text-gray-500 capitalize">{entity.entityType.replace('_', ' ')}</p>
          {entity.extractionConfidence && (
            <div className="flex items-center gap-1 mt-1">
              <div className="h-1 w-16 rounded-full bg-gray-700 overflow-hidden">
                <div
                  className="h-full rounded-full bg-blue-500"
                  style={{ width: `${entity.extractionConfidence * 100}%` }}
                />
              </div>
              <span className="text-xs text-gray-500">{Math.round(entity.extractionConfidence * 100)}%</span>
            </div>
          )}
        </div>
      </div>
      {/* Arrow */}
      <div className="absolute left-1/2 -translate-x-1/2 top-full w-0 h-0 
                      border-l-[6px] border-l-transparent 
                      border-r-[6px] border-r-transparent 
                      border-t-[6px] border-t-gray-800" />
    </div>
  );
}

export function EntityMentions({ transcript, entities, onMentionClick }: EntityMentionsProps) {
  const [hoveredEntity, setHoveredEntity] = useState<Entity | null>(null);
  const [tooltipPosition, setTooltipPosition] = useState({ x: 0, y: 0 });

  const mentions = useMemo(() => {
    return findMentions(transcript, entities);
  }, [transcript, entities]);

  const handleMouseEnter = useCallback((entity: Entity, e: React.MouseEvent) => {
    setHoveredEntity(entity);
    setTooltipPosition({ x: e.clientX, y: e.clientY });
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    setTooltipPosition({ x: e.clientX, y: e.clientY });
  }, []);

  const handleMouseLeave = useCallback(() => {
    setHoveredEntity(null);
  }, []);

  const handleClick = useCallback((entity: Entity, e: React.MouseEvent) => {
    e.preventDefault();
    onMentionClick?.(entity);
  }, [onMentionClick]);

  const handleScrollToMention = useCallback((mention: Mention) => {
    // This would be implemented with refs in a real app
    console.log('Scroll to mention:', mention);
  }, []);

  // Build the highlighted transcript
  const renderTranscript = useMemo(() => {
    if (mentions.length === 0) {
      return <span className="text-gray-300">{transcript}</span>;
    }

    const elements: React.ReactNode[] = [];
    let lastIndex = 0;

    mentions.forEach((mention, index) => {
      // Add text before mention
      if (mention.startIndex > lastIndex) {
        elements.push(
          <span key={`text-${index}`} className="text-gray-300">
            {transcript.slice(lastIndex, mention.startIndex)}
          </span>
        );
      }

      // Add highlighted mention
      const highlightColor = entityHighlightColors[mention.entity.entityType] || entityHighlightColors[EntityType.Concept];
      
      elements.push(
        <mark
          key={`mention-${index}`}
          onClick={(e) => handleClick(mention.entity, e)}
          onMouseEnter={(e) => handleMouseEnter(mention.entity, e)}
          onMouseMove={handleMouseMove}
          onMouseLeave={handleMouseLeave}
          className={`
            inline cursor-pointer rounded px-0.5 border
            transition-colors duration-150
            ${highlightColor}
          `}
        >
          {mention.text}
        </mark>
      );

      lastIndex = mention.endIndex;
    });

    // Add remaining text
    if (lastIndex < transcript.length) {
      elements.push(
        <span key="text-end" className="text-gray-300">
          {transcript.slice(lastIndex)}
        </span>
      );
    }

    return elements;
  }, [mentions, transcript, handleClick, handleMouseEnter, handleMouseMove, handleMouseLeave]);

  // Group mentions by entity for the sidebar
  const mentionsByEntity = useMemo(() => {
    const map = new Map<string, { entity: Entity; count: number; mentions: Mention[] }>();
    
    mentions.forEach((mention) => {
      const existing = map.get(mention.entity.id);
      if (existing) {
        existing.count++;
        existing.mentions.push(mention);
      } else {
        map.set(mention.entity.id, { entity: mention.entity, count: 1, mentions: [mention] });
      }
    });
    
    return Array.from(map.values()).sort((a, b) => b.count - a.count);
  }, [mentions]);

  return (
    <div className="flex gap-6">
      {/* Transcript with highlights */}
      <div className="flex-1">
        <div className="bg-gray-800/30 border border-gray-700 rounded-lg p-4">
          <pre className="font-mono text-sm leading-relaxed whitespace-pre-wrap">
            {renderTranscript}
          </pre>
        </div>
      </div>

      {/* Mention sidebar */}
      {mentionsByEntity.length > 0 && (
        <div className="w-64 flex-shrink-0">
          <h4 className="text-sm font-medium text-gray-400 uppercase tracking-wide mb-3">
            Mentioned Entities
          </h4>
          <div className="space-y-2">
            {mentionsByEntity.map(({ entity, count, mentions: entityMentions }) => (
              <div
                key={entity.id}
                className="
                  p-2 rounded-lg border border-gray-700 bg-gray-800/30
                  hover:border-gray-600 hover:bg-gray-800/50
                  transition-all cursor-pointer group
                "
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 min-w-0">
                    <div className={`
                      w-2 h-2 rounded-full flex-shrink-0
                      ${entityHighlightColors[entity.entityType].split(' ')[0].replace('bg-', 'bg-').replace('/20', '')}
                    `} />
                    <span className="text-sm text-gray-300 truncate">{entity.name}</span>
                  </div>
                  <span className="text-xs text-gray-500 flex-shrink-0">{count}×</span>
                </div>
                
                {/* Quick jump buttons */}
                <div className="flex items-center gap-1 mt-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                  {entityMentions.slice(0, 3).map((mention, idx) => (
                    <button
                      key={idx}
                      onClick={() => handleScrollToMention(mention)}
                      className="
                        flex items-center gap-1 px-1.5 py-0.5 
                        text-xs text-gray-500 bg-gray-700/50 rounded
                        hover:text-gray-300 hover:bg-gray-700
                        transition-colors
                      "
                      title={`Jump to mention ${idx + 1}`}
                    >
                      <ArrowUpRight size={10} />
                      {idx + 1}
                    </button>
                  ))}
                  {entityMentions.length > 3 && (
                    <span className="text-xs text-gray-600">+{entityMentions.length - 3}</span>
                  )}
                </div>
              </div>
            ))}
          </div>

          {/* Legend */}
          <div className="mt-4 pt-4 border-t border-gray-800">
            <h5 className="text-xs text-gray-500 mb-2">Legend</h5>
            <div className="space-y-1">
              {Object.entries(entityHighlightColors).slice(0, 4).map(([type, colorClass]) => (
                <div key={type} className="flex items-center gap-2">
                  <span className={`w-3 h-3 rounded ${colorClass.split(' ')[0]}`} />
                  <span className="text-xs text-gray-500 capitalize">{type.replace('_', ' ')}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Tooltip */}
      <EntityTooltip
        entity={hoveredEntity!}
        isVisible={!!hoveredEntity}
        position={tooltipPosition}
      />
    </div>
  );
}

export default EntityMentions;
