import { 
  User, 
  Building2, 
  MapPin, 
  Calendar, 
  Package, 
  Lightbulb, 
  Scale, 
  CheckSquare,
  type LucideIcon
} from 'lucide-react';
import { EntityType } from '../../types/knowledge-graph';

export interface EntityTypeBadgeProps {
  type: EntityType;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
}

const typeConfig: Record<EntityType, { color: string; icon: LucideIcon; label: string }> = {
  [EntityType.Person]: {
    color: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
    icon: User,
    label: 'Person',
  },
  [EntityType.Organization]: {
    color: 'bg-green-500/20 text-green-400 border-green-500/30',
    icon: Building2,
    label: 'Organization',
  },
  [EntityType.Location]: {
    color: 'bg-orange-500/20 text-orange-400 border-orange-500/30',
    icon: MapPin,
    label: 'Location',
  },
  [EntityType.Event]: {
    color: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
    icon: Calendar,
    label: 'Event',
  },
  [EntityType.Product]: {
    color: 'bg-pink-500/20 text-pink-400 border-pink-500/30',
    icon: Package,
    label: 'Product',
  },
  [EntityType.Concept]: {
    color: 'bg-gray-500/20 text-gray-400 border-gray-500/30',
    icon: Lightbulb,
    label: 'Concept',
  },
  [EntityType.Decision]: {
    color: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
    icon: Scale,
    label: 'Decision',
  },
  [EntityType.ActionItem]: {
    color: 'bg-red-500/20 text-red-400 border-red-500/30',
    icon: CheckSquare,
    label: 'Action Item',
  },
};

const sizeConfig = {
  sm: {
    container: 'px-1.5 py-0.5 text-xs gap-1',
    icon: 12,
  },
  md: {
    container: 'px-2 py-1 text-sm gap-1.5',
    icon: 14,
  },
  lg: {
    container: 'px-3 py-1.5 text-base gap-2',
    icon: 18,
  },
};

export function EntityTypeBadge({ type, size = 'md', showLabel = true }: EntityTypeBadgeProps) {
  const config = typeConfig[type];
  const sizeStyles = sizeConfig[size];
  const Icon = config.icon;

  return (
    <span
      className={`
        inline-flex items-center rounded-lg border font-medium
        ${config.color}
        ${sizeStyles.container}
      `}
      title={config.label}
    >
      <Icon size={sizeStyles.icon} />
      {showLabel && <span>{config.label}</span>}
    </span>
  );
}

// Helper to get just the icon for compact displays
export function EntityTypeIcon({ type, size = 16 }: { type: EntityType; size?: number }) {
  const config = typeConfig[type];
  const Icon = config.icon;
  
  return (
    <span className={`inline-flex items-center justify-center ${config.color.split(' ')[1]}`}>
      <Icon size={size} />
    </span>
  );
}

// Helper to get color class for custom styling
export function getEntityTypeColor(type: EntityType): string {
  return typeConfig[type].color;
}

export default EntityTypeBadge;
