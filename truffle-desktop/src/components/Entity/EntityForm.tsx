import { useState, useEffect } from 'react';
import { 
  User, 
  Building2, 
  MapPin, 
  Calendar, 
  Package, 
  Lightbulb, 
  Scale, 
  CheckSquare,
  X,
  Save,
  AlertCircle
} from 'lucide-react';
import { EntityTypeBadge } from './EntityTypeBadge';
import type { 
  Entity, 
  EntityType, 
  CreateEntityInput, 
  UpdateEntityInput,
  EntityMetadata 
} from '../../types/knowledge-graph';

export interface EntityFormProps {
  entity?: Entity;
  onSubmit: (data: CreateEntityInput | UpdateEntityInput) => void;
  onCancel?: () => void;
  isSubmitting?: boolean;
}

type FormErrors = {
  name?: string;
  type?: string;
  description?: string;
} & Partial<Record<keyof EntityMetadata, string>>;

const entityTypeOptions = [
  { value: EntityType.Person, icon: User, label: 'Person' },
  { value: EntityType.Organization, icon: Building2, label: 'Organization' },
  { value: EntityType.Location, icon: MapPin, label: 'Location' },
  { value: EntityType.Event, icon: Calendar, label: 'Event' },
  { value: EntityType.Product, icon: Package, label: 'Product' },
  { value: EntityType.Concept, icon: Lightbulb, label: 'Concept' },
  { value: EntityType.Decision, icon: Scale, label: 'Decision' },
  { value: EntityType.ActionItem, icon: CheckSquare, label: 'Action Item' },
];

export function EntityForm({ entity, onSubmit, onCancel, isSubmitting = false }: EntityFormProps) {
  const isEditMode = !!entity;
  
  const [name, setName] = useState(entity?.name ?? '');
  const [type, setType] = useState<EntityType>(entity?.entityType ?? EntityType.Person);
  const [description, setDescription] = useState(entity?.description ?? '');
  const [metadata, setMetadata] = useState<Partial<EntityMetadata>>(entity?.metadata ?? {});
  const [errors, setErrors] = useState<FormErrors>({});
  const [touched, setTouched] = useState<Record<string, boolean>>({});

  // Reset form when entity changes
  useEffect(() => {
    if (entity) {
      setName(entity.name);
      setType(entity.entityType);
      setDescription(entity.description ?? '');
      setMetadata(entity.metadata ?? {});
    }
  }, [entity?.id]);

  const validate = (): boolean => {
    const newErrors: FormErrors = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    } else if (name.length < 2) {
      newErrors.name = 'Name must be at least 2 characters';
    }

    if (!isEditMode && !type) {
      newErrors.type = 'Type is required';
    }

    // Type-specific validation
    if (type === EntityType.Person) {
      if (metadata.email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(metadata.email)) {
        newErrors.email = 'Invalid email address';
      }
    }

    if (type === EntityType.Organization) {
      if (metadata.foundedYear && (metadata.foundedYear < 1000 || metadata.foundedYear > new Date().getFullYear())) {
        newErrors.foundedYear = `Year must be between 1000 and ${new Date().getFullYear()}`;
      }
    }

    if (type === EntityType.Location) {
      if (metadata.coordinates) {
        const [lat, lng] = metadata.coordinates;
        if (lat < -90 || lat > 90 || lng < -180 || lng > 180) {
          newErrors.coordinates = 'Invalid coordinates';
        }
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  useEffect(() => {
    if (Object.keys(touched).length > 0) {
      validate();
    }
  }, [name, type, description, metadata, touched]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Mark all fields as touched
    setTouched({ name: true, type: true, description: true });

    if (!validate()) {
      return;
    }

    const formData: CreateEntityInput | UpdateEntityInput = isEditMode
      ? {
          name: name.trim(),
          description: description.trim() || undefined,
          metadata: Object.keys(metadata).length > 0 ? metadata : undefined,
        }
      : {
          entityType: type,
          name: name.trim(),
          description: description.trim() || undefined,
          metadata: Object.keys(metadata).length > 0 ? metadata : undefined,
        };

    onSubmit(formData);
  };

  const handleMetadataChange = (key: keyof EntityMetadata, value: unknown) => {
    setMetadata(prev => ({ ...prev, [key]: value }));
    setTouched(prev => ({ ...prev, [key]: true }));
  };

  const renderTypeSpecificFields = () => {
    switch (type) {
      case EntityType.Person:
        return (
          <>
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-300">Role</label>
              <input
                type="text"
                value={metadata.role ?? ''}
                onChange={(e) => handleMetadataChange('role', e.target.value || undefined)}
                placeholder="e.g., Software Engineer"
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-300">Email</label>
              <input
                type="email"
                value={metadata.email ?? ''}
                onChange={(e) => handleMetadataChange('email', e.target.value || undefined)}
                placeholder="person@example.com"
                className={`w-full px-3 py-2 bg-gray-800 border rounded-lg text-gray-200 placeholder-gray-500 outline-none transition-colors ${
                  errors.email ? 'border-red-500 focus:border-red-500 focus:ring-1 focus:ring-red-500' : 'border-gray-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500'
                }`}
              />
              {errors.email && (
                <p className="text-sm text-red-400 flex items-center gap-1">
                  <AlertCircle size={14} /> {errors.email}
                </p>
              )}
            </div>
          </>
        );

      case EntityType.Organization:
        return (
          <>
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-300">Industry</label>
              <input
                type="text"
                value={metadata.industry ?? ''}
                onChange={(e) => handleMetadataChange('industry', e.target.value || undefined)}
                placeholder="e.g., Technology"
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-300">Founded Year</label>
              <input
                type="number"
                value={metadata.foundedYear ?? ''}
                onChange={(e) => handleMetadataChange('foundedYear', e.target.value ? parseInt(e.target.value) : undefined)}
                placeholder="e.g., 2020"
                className={`w-full px-3 py-2 bg-gray-800 border rounded-lg text-gray-200 placeholder-gray-500 outline-none transition-colors ${
                  errors.foundedYear ? 'border-red-500 focus:border-red-500 focus:ring-1 focus:ring-red-500' : 'border-gray-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500'
                }`}
              />
              {errors.foundedYear && (
                <p className="text-sm text-red-400 flex items-center gap-1">
                  <AlertCircle size={14} /> {errors.foundedYear}
                </p>
              )}
            </div>
          </>
        );

      case EntityType.Location:
        return (
          <div className="space-y-2">
            <label className="text-sm font-medium text-gray-300">Coordinates</label>
            <div className="flex gap-2">
              <input
                type="number"
                step="any"
                value={metadata.coordinates?.[0] ?? ''}
                onChange={(e) => {
                  const lat = e.target.value ? parseFloat(e.target.value) : 0;
                  const lng = metadata.coordinates?.[1] ?? 0;
                  handleMetadataChange('coordinates', e.target.value ? [lat, lng] : undefined);
                }}
                placeholder="Latitude"
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
              />
              <input
                type="number"
                step="any"
                value={metadata.coordinates?.[1] ?? ''}
                onChange={(e) => {
                  const lat = metadata.coordinates?.[0] ?? 0;
                  const lng = e.target.value ? parseFloat(e.target.value) : 0;
                  handleMetadataChange('coordinates', e.target.value ? [lat, lng] : undefined);
                }}
                placeholder="Longitude"
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
              />
            </div>
            {errors.coordinates && (
              <p className="text-sm text-red-400 flex items-center gap-1">
                <AlertCircle size={14} /> {errors.coordinates}
              </p>
            )}
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-gray-700">
        <h2 className="text-xl font-semibold text-gray-100">
          {isEditMode ? 'Edit Entity' : 'Create Entity'}
        </h2>
        {isEditMode && entity && (
          <EntityTypeBadge type={entity.entityType} />
        )}
      </div>

      {/* Basic Fields */}
      <div className="space-y-4">
        {!isEditMode && (
          <div className="space-y-2">
            <label className="text-sm font-medium text-gray-300">
              Entity Type <span className="text-red-400">*</span>
            </label>
            <div className="grid grid-cols-4 gap-2">
              {entityTypeOptions.map((option) => {
                const Icon = option.icon;
                return (
                  <button
                    key={option.value}
                    type="button"
                    onClick={() => setType(option.value)}
                    className={`
                      flex flex-col items-center gap-1 p-3 rounded-lg border transition-all
                      ${type === option.value
                        ? 'border-blue-500 bg-blue-500/20 text-blue-400'
                        : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600 hover:bg-gray-750'
                      }
                    `}
                  >
                    <Icon size={20} />
                    <span className="text-xs">{option.label}</span>
                  </button>
                );
              })}
            </div>
          </div>
        )}

        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-300">
            Name <span className="text-red-400">*</span>
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onBlur={() => setTouched(prev => ({ ...prev, name: true }))}
            placeholder="Entity name"
            className={`w-full px-3 py-2 bg-gray-800 border rounded-lg text-gray-200 placeholder-gray-500 outline-none transition-colors ${
              errors.name && touched.name
                ? 'border-red-500 focus:border-red-500 focus:ring-1 focus:ring-red-500'
                : 'border-gray-700 focus:border-blue-500 focus:ring-1 focus:ring-blue-500'
            }`}
          />
          {errors.name && touched.name && (
            <p className="text-sm text-red-400 flex items-center gap-1">
              <AlertCircle size={14} /> {errors.name}
            </p>
          )}
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-300">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Optional description"
            rows={3}
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors resize-none"
          />
        </div>
      </div>

      {/* Type-specific Metadata */}
      <div className="space-y-4 pt-4 border-t border-gray-700">
        <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide">
          Type-specific Fields
        </h3>
        {renderTypeSpecificFields()}
      </div>

      {/* Common Metadata */}
      <div className="space-y-4 pt-4 border-t border-gray-700">
        <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide">
          Additional Information
        </h3>
        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-300">Tags (comma-separated)</label>
          <input
            type="text"
            value={metadata.tags?.join(', ') ?? ''}
            onChange={(e) => handleMetadataChange('tags', e.target.value ? e.target.value.split(',').map(t => t.trim()).filter(Boolean) : undefined)}
            placeholder="tag1, tag2, tag3"
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
          />
        </div>
        <div className="space-y-2">
          <label className="text-sm font-medium text-gray-300">Aliases (comma-separated)</label>
          <input
            type="text"
            value={metadata.aliases?.join(', ') ?? ''}
            onChange={(e) => handleMetadataChange('aliases', e.target.value ? e.target.value.split(',').map(t => t.trim()).filter(Boolean) : undefined)}
            placeholder="alternative name 1, alternative name 2"
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
          />
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center justify-end gap-3 pt-4 border-t border-gray-700">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            disabled={isSubmitting}
            className="flex items-center gap-2 px-4 py-2 text-gray-300 hover:text-gray-100 hover:bg-gray-800 rounded-lg transition-colors disabled:opacity-50"
          >
            <X size={18} />
            Cancel
          </button>
        )}
        <button
          type="submit"
          disabled={isSubmitting || Object.keys(errors).length > 0}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Save size={18} />
          {isSubmitting ? 'Saving...' : isEditMode ? 'Update Entity' : 'Create Entity'}
        </button>
      </div>
    </form>
  );
}

export default EntityForm;
