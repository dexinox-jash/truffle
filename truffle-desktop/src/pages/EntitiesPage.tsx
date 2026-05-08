import React, { useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Plus, Search, LayoutGrid, List as ListIcon, Filter, X } from 'lucide-react';
import { useEntityStore } from '../store/entityStore';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { EntityList } from '../components/Entity/EntityList';
import { EntityDetail } from '../components/Entity/EntityDetail';
import { EntityForm } from '../components/Entity/EntityForm';
import { EmptyState } from '../components/ui/EmptyState';
import { Database } from 'lucide-react';

export const EntitiesPage: React.FC = () => {
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();

  const {
    viewMode,
    setViewMode,
    selectedEntityId,
    setSelectedEntityId,
    listFilter,
    setListFilter,
    resetListFilter,
    openEntityDetail,
    openEntityCreate,
    closeDetail,
  } = useEntityStore();

  // Sync URL params with store
  useEffect(() => {
    if (id) {
      setSelectedEntityId(id);
      setViewMode('detail');
    }
  }, [id, setSelectedEntityId, setViewMode]);

  const handleCreate = useCallback(() => {
    openEntityCreate();
    navigate('/entities');
  }, [openEntityCreate, navigate]);

  const handleSelect = useCallback(
    (entityId: string) => {
      openEntityDetail(entityId);
      navigate(`/entities/${entityId}`);
    },
    [openEntityDetail, navigate]
  );

  const handleCloseDetail = useCallback(() => {
    closeDetail();
    navigate('/entities');
  }, [closeDetail, navigate]);

  const handleSearchChange = useCallback(
    (value: string) => {
      setListFilter({ ...listFilter, search: value });
    },
    [listFilter, setListFilter]
  );

  const hasActiveFilters =
    listFilter.search ||
    listFilter.types.length > 0 ||
    listFilter.tags.length > 0 ||
    listFilter.dateRange;

  return (
    <div className="h-[calc(100vh-4rem)] flex bg-darkroom-bg">
      {/* Sidebar */}
      <div className="w-80 border-r border-darkroom-gray-700 flex flex-col bg-darkroom-card">
        {/* Header */}
        <div className="p-4 border-b border-darkroom-gray-700">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-darkroom-text-primary">
              Entities
            </h2>
            <Button
              size="sm"
              onClick={handleCreate}
              leftIcon={<Plus className="w-4 h-4" />}
            >
              Create
            </Button>
          </div>

          {/* Search */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-darkroom-text-tertiary" />
            <Input
              placeholder="Search entities..."
              value={listFilter.search}
              onChange={(e) => handleSearchChange(e.target.value)}
              className="pl-9 pr-9"
            />
            {listFilter.search && (
              <button
                onClick={() => handleSearchChange('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-darkroom-text-tertiary hover:text-darkroom-text-primary"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>

          {/* Filter bar */}
          <div className="flex items-center justify-between mt-3">
            <Button
              variant="ghost"
              size="sm"
              leftIcon={<Filter className="w-3.5 h-3.5" />}
              className={hasActiveFilters ? 'text-accent' : ''}
            >
              Filters
            </Button>
            {hasActiveFilters && (
              <button
                onClick={resetListFilter}
                className="text-xs text-darkroom-text-tertiary hover:text-darkroom-text-primary"
              >
                Reset
              </button>
            )}
          </div>
        </div>

        {/* Entity List */}
        <div className="flex-1 overflow-hidden">
          <EntityList
            filter={listFilter}
            selectedId={selectedEntityId}
            onSelect={handleSelect}
          />
        </div>

        {/* Footer stats */}
        <div className="p-3 border-t border-darkroom-gray-700 text-xs text-darkroom-text-tertiary">
          <div className="flex justify-between">
            <span>Total: 0 entities</span>
            <span>Filtered: 0</span>
          </div>
        </div>
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-auto">
        {viewMode === 'list' && (
          <EmptyState
            icon={<Database className="w-16 h-16" />}
            title="No Entity Selected"
            description="Select an entity from the list to view details, or create a new one to get started."
            action={
              <Button onClick={handleCreate} leftIcon={<Plus className="w-4 h-4" />}>
                Create Entity
              </Button>
            }
          />
        )}

        {viewMode === 'detail' && selectedEntityId && (
          <EntityDetail
            entityId={selectedEntityId}
            onEdit={() => setViewMode('edit')}
            onClose={handleCloseDetail}
          />
        )}

        {viewMode === 'edit' && selectedEntityId && (
          <div className="max-w-4xl mx-auto p-6">
            <div className="flex items-center justify-between mb-6">
              <h1 className="text-2xl font-bold text-darkroom-text-primary">
                Edit Entity
              </h1>
              <Button variant="ghost" onClick={() => setViewMode('detail')}>
                Cancel
              </Button>
            </div>
            <EntityForm
              entityId={selectedEntityId}
              onSubmit={() => setViewMode('detail')}
              onCancel={() => setViewMode('detail')}
            />
          </div>
        )}

        {viewMode === 'create' && (
          <div className="max-w-4xl mx-auto p-6">
            <div className="flex items-center justify-between mb-6">
              <h1 className="text-2xl font-bold text-darkroom-text-primary">
                Create Entity
              </h1>
              <Button variant="ghost" onClick={handleCloseDetail}>
                Cancel
              </Button>
            </div>
            <EntityForm
              onSubmit={() => navigate('/entities')}
              onCancel={handleCloseDetail}
            />
          </div>
        )}
      </div>
    </div>
  );
};

export default EntitiesPage;
