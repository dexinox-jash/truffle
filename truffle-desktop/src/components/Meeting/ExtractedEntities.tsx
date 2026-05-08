import { useMemo, useState } from 'react';
import { 
  User, 
  Building2, 
  MapPin, 
  Scale, 
  CheckSquare,
  Network,
  ArrowRight,
  Plus,
  CheckCircle2,
} from 'lucide-react';
import { EntityTypeBadge } from '../Entity/EntityTypeBadge';
import type { Entity, Relationship } from '../../types/knowledge-graph';
import { EntityType } from '../../types/knowledge-graph';

export interface ExtractedEntitiesProps {
  meetingId: string;
  entities: Entity[];
  relationships: Relationship[];
  onEntityClick?: (entity: Entity) => void;
  onAddToGraph?: (entityId: string) => void;
}

interface GroupedEntities {
  people: Entity[];
  organizations: Entity[];
  locations: Entity[];
  decisions: Entity[];
  actionItems: Entity[];
  other: Entity[];
}

function groupEntities(entities: Entity[]): GroupedEntities {
  return entities.reduce((acc, entity) => {
    switch (entity.entityType) {
      case EntityType.Person:
        acc.people.push(entity);
        break;
      case EntityType.Organization:
        acc.organizations.push(entity);
        break;
      case EntityType.Location:
        acc.locations.push(entity);
        break;
      case EntityType.Decision:
        acc.decisions.push(entity);
        break;
      case EntityType.ActionItem:
        acc.actionItems.push(entity);
        break;
      default:
        acc.other.push(entity);
    }
    return acc;
  }, {
    people: [],
    organizations: [],
    locations: [],
    decisions: [],
    actionItems: [],
    other: [],
  } as GroupedEntities);
}

function ConfidenceBadge({ confidence }: { confidence: number }) {
  let colorClass = 'bg-gray-500';
  let label = 'Low';
  
  if (confidence >= 0.9) {
    colorClass = 'bg-green-500';
    label = 'High';
  } else if (confidence >= 0.7) {
    colorClass = 'bg-yellow-500';
    label = 'Good';
  } else if (confidence >= 0.5) {
    colorClass = 'bg-orange-500';
    label = 'Fair';
  }

  return (
    <div className="flex items-center gap-1.5" title={`Confidence: ${Math.round(confidence * 100)}%`}>
      <div className="h-1.5 w-12 rounded-full bg-gray-700 overflow-hidden">
        <div
          className={`h-full rounded-full ${colorClass} transition-all duration-300`}
          style={{ width: `${confidence * 100}%` }}
        />
      </div>
      <span className="text-xs text-gray-500">{Math.round(confidence * 100)}%</span>
    </div>
  );
}

function EntityItem({ 
  entity, 
  onClick, 
  onAddToGraph,
  isAdded = false,
}: { 
  entity: Entity; 
  onClick?: () => void;
  onAddToGraph?: () => void;
  isAdded?: boolean;
}) {
  return (
    <div
      className="
        p-3 rounded-lg border border-gray-700 bg-gray-800/30
        hover:border-gray-600 hover:bg-gray-800/50
        transition-all group
      "
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-2.5 flex-1 min-w-0">
          <EntityTypeBadge type={entity.entityType} size="sm" showLabel={false} />
          <div className="flex-1 min-w-0">
            <h4 
              onClick={onClick}
              className="font-medium text-gray-200 truncate cursor-pointer hover:text-blue-400 transition-colors"
            >
              {entity.name}
            </h4>
            {entity.description && (
              <p className="text-xs text-gray-500 line-clamp-1 mt-0.5">{entity.description}</p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <ConfidenceBadge confidence={entity.extractionConfidence} />
          {onAddToGraph && (
            <button
              onClick={onAddToGraph}
              disabled={isAdded}
              className={`
                p-1.5 rounded transition-colors
                ${isAdded 
                  ? 'text-green-400 cursor-default' 
                  : 'text-gray-400 hover:text-blue-400 hover:bg-blue-500/10 opacity-0 group-hover:opacity-100'
                }
              `}
              title={isAdded ? 'Added to graph' : 'Add to graph'}
            >
              {isAdded ? <CheckCircle2 size={16} /> : <Plus size={16} />}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function EntitySection({ 
  title, 
  icon: Icon, 
  entities, 
  onEntityClick, 
  onAddToGraph,
  addedIds = [],
}: { 
  title: string; 
  icon: typeof User; 
  entities: Entity[];
  onEntityClick?: (entity: Entity) => void;
  onAddToGraph?: (entityId: string) => void;
  addedIds?: string[];
}) {
  if (entities.length === 0) return null;

  return (
    <div className="mb-6">
      <div className="flex items-center gap-2 mb-3">
        <Icon size={16} className="text-gray-500" />
        <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide">{title}</h3>
        <span className="text-xs text-gray-600">({entities.length})</span>
      </div>
      <div className="space-y-2">
        {entities.map((entity) => (
          <EntityItem
            key={entity.id}
            entity={entity}
            onClick={() => onEntityClick?.(entity)}
            onAddToGraph={onAddToGraph ? () => onAddToGraph(entity.id) : undefined}
            isAdded={addedIds.includes(entity.id)}
          />
        ))}
      </div>
    </div>
  );
}

function RelationshipPreview({ 
  relationships, 
  entities 
}: { 
  relationships: Relationship[]; 
  entities: Entity[];
}) {
  const entityMap = useMemo(() => {
    return entities.reduce((acc, entity) => {
      acc[entity.id] = entity;
      return acc;
    }, {} as Record<string, Entity>);
  }, [entities]);

  if (relationships.length === 0) return null;

  return (
    <div className="mb-6">
      <div className="flex items-center gap-2 mb-3">
        <Network size={16} className="text-gray-500" />
        <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide">Relationships</h3>
        <span className="text-xs text-gray-600">({relationships.length})</span>
      </div>
      <div className="space-y-2">
        {relationships.slice(0, 5).map((rel) => {
          const source = entityMap[rel.sourceId];
          const target = entityMap[rel.targetId];
          if (!source || !target) return null;

          return (
            <div 
              key={rel.id} 
              className="flex items-center gap-2 p-2 bg-gray-800/30 rounded-lg text-sm"
            >
              <span className="text-gray-300 truncate max-w-[120px]">{source.name}</span>
              <ArrowRight size={14} className="text-gray-500 flex-shrink-0" />
              <span className="text-blue-400 capitalize flex-shrink-0">{rel.relationType}</span>
              <ArrowRight size={14} className="text-gray-500 flex-shrink-0" />
              <span className="text-gray-300 truncate max-w-[120px]">{target.name}</span>
            </div>
          );
        })}
        {relationships.length > 5 && (
          <p className="text-sm text-gray-500 text-center">
            +{relationships.length - 5} more relationships
          </p>
        )}
      </div>
    </div>
  );
}

export function ExtractedEntities({ 
  meetingId, 
  entities, 
  relationships, 
  onEntityClick, 
  onAddToGraph 
}: ExtractedEntitiesProps) {
  const grouped = useMemo(() => groupEntities(entities), [entities]);
  const [addedIds, setAddedIds] = useState<string[]>([]);

  const handleAddAll = () => {
    const allIds = entities.map(e => e.id);
    setAddedIds(allIds);
    allIds.forEach(id => onAddToGraph?.(id));
  };

  const handleAddOne = (id: string) => {
    setAddedIds(prev => [...prev, id]);
    onAddToGraph?.(id);
  };

  if (entities.length === 0) {
    return (
      <div className="text-center text-gray-500 py-8">
        <Network size={48} className="mx-auto mb-4 text-gray-600" />
        <p>No entities found in this meeting.</p>
      </div>
    );
  }

  return (
    <div>
      {/* Header with Add All button */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h3 className="text-lg font-medium text-gray-200">Extracted Entities</h3>
          <p className="text-sm text-gray-500">
            {entities.length} entities found with {relationships.length} relationships
          </p>
        </div>
        {onAddToGraph && (
          <button
            onClick={handleAddAll}
            disabled={addedIds.length === entities.length}
            className="
              flex items-center gap-2 px-4 py-2 
              bg-blue-500/10 text-blue-400 border border-blue-500/20
              rounded-lg hover:bg-blue-500 hover:text-white hover:border-blue-500
              transition-all disabled:opacity-50 disabled:cursor-not-allowed
            "
          >
            <Plus size={16} />
            <span>Add All to Graph</span>
          </button>
        )}
      </div>

      {/* Entity Sections */}
      <EntitySection
        title="People"
        icon={User}
        entities={grouped.people}
        onEntityClick={onEntityClick}
        onAddToGraph={onAddToGraph ? handleAddOne : undefined}
        addedIds={addedIds}
      />
      <EntitySection
        title="Organizations"
        icon={Building2}
        entities={grouped.organizations}
        onEntityClick={onEntityClick}
        onAddToGraph={onAddToGraph ? handleAddOne : undefined}
        addedIds={addedIds}
      />
      <EntitySection
        title="Locations"
        icon={MapPin}
        entities={grouped.locations}
        onEntityClick={onEntityClick}
        onAddToGraph={onAddToGraph ? handleAddOne : undefined}
        addedIds={addedIds}
      />
      <EntitySection
        title="Decisions"
        icon={Scale}
        entities={grouped.decisions}
        onEntityClick={onEntityClick}
        onAddToGraph={onAddToGraph ? handleAddOne : undefined}
        addedIds={addedIds}
      />
      <EntitySection
        title="Action Items"
        icon={CheckSquare}
        entities={grouped.actionItems}
        onEntityClick={onEntityClick}
        onAddToGraph={onAddToGraph ? handleAddOne : undefined}
        addedIds={addedIds}
      />
      <EntitySection
        title="Other"
        icon={Network}
        entities={grouped.other}
        onEntityClick={onEntityClick}
        onAddToGraph={onAddToGraph ? handleAddOne : undefined}
        addedIds={addedIds}
      />

      {/* Relationships */}
      <RelationshipPreview relationships={relationships} entities={entities} />

      {/* View in Graph link */}
      <div className="mt-6 pt-4 border-t border-gray-800">
        <button
          className="flex items-center gap-2 text-sm text-blue-400 hover:text-blue-300 transition-colors"
        >
          <Network size={16} />
          <span>View in Knowledge Graph</span>
          <ArrowRight size={14} />
        </button>
      </div>
    </div>
  );
}

export default ExtractedEntities;
