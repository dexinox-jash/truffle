import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  ShieldAlert,
  AlertTriangle,
  Copy,
  CheckCircle,
  AlertCircle,
  ChevronRight,
  Merge,
  X,
} from 'lucide-react';
import { useEntityStore } from '../store/entityStore';
import { Button } from '../components/ui/Button';
import { Tabs, TabList, Tab, TabPanels, TabPanel } from '../components/ui/Tabs';
import { ValidationSummary } from '../components/Validation/ValidationSummary';
import { ContradictionList } from '../components/Validation/ContradictionList';
import { DuplicateList } from '../components/Validation/DuplicateList';
import { MergeModal } from '../components/Validation/MergeModal';

export const ValidationPage: React.FC = () => {
  const navigate = useNavigate();
  const {
    validationTab,
    setValidationTab,
    selectedContradictionId,
    setSelectedContradictionId,
    selectedDuplicateIds,
    setSelectedDuplicateIds,
  } = useEntityStore();

  const [showMergeModal, setShowMergeModal] = useState(false);

  // Mock data for demonstration
  const summaryStats = {
    totalEntities: 156,
    contradictions: 12,
    duplicates: 8,
    completeness: 87,
    lastValidated: new Date(),
  };

  const handleTabChange = (tab: 'summary' | 'contradictions' | 'duplicates') => {
    setValidationTab(tab);
    // Clear selections when switching tabs
    setSelectedContradictionId(null);
    setSelectedDuplicateIds(null);
  };

  const handleMerge = (sourceId: string, targetId: string) => {
    setSelectedDuplicateIds([sourceId, targetId]);
    setShowMergeModal(true);
  };

  return (
    <div className="h-[calc(100vh-4rem)] flex bg-darkroom-bg">
      {/* Main content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-darkroom-gray-700 bg-darkroom-card">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-darkroom-text-primary flex items-center gap-2">
                <ShieldAlert className="w-6 h-6 text-amber-500" />
                Validation
              </h1>
              <p className="text-sm text-darkroom-text-secondary mt-1">
                Review and resolve data quality issues
              </p>
            </div>
            <Button
              variant="secondary"
              leftIcon={<CheckCircle className="w-4 h-4" />}
            >
              Run Validation
            </Button>
          </div>

          {/* Summary stats */}
          <div className="grid grid-cols-4 gap-4 mt-4">
            <StatCard
              label="Total Entities"
              value={summaryStats.totalEntities}
              icon={<CheckCircle className="w-4 h-4 text-green-500" />}
            />
            <StatCard
              label="Contradictions"
              value={summaryStats.contradictions}
              icon={<AlertTriangle className="w-4 h-4 text-red-500" />}
              status={summaryStats.contradictions > 0 ? 'error' : 'success'}
            />
            <StatCard
              label="Duplicates"
              value={summaryStats.duplicates}
              icon={<Copy className="w-4 h-4 text-amber-500" />}
              status={summaryStats.duplicates > 0 ? 'warning' : 'success'}
            />
            <StatCard
              label="Completeness"
              value={`${summaryStats.completeness}%`}
              icon={<CheckCircle className="w-4 h-4 text-blue-500" />}
            />
          </div>
        </div>

        {/* Tabs */}
        <div className="flex-1 overflow-hidden">
          <Tabs
            value={validationTab}
            onChange={(value) =>
              handleTabChange(value as 'summary' | 'contradictions' | 'duplicates')
            }
          >
            <TabList className="px-6 border-b border-darkroom-gray-700">
              <Tab value="summary">Summary</Tab>
              <Tab value="contradictions">
                Contradictions
                {summaryStats.contradictions > 0 && (
                  <span className="ml-2 px-2 py-0.5 text-xs bg-red-500/20 text-red-400 rounded-full">
                    {summaryStats.contradictions}
                  </span>
                )}
              </Tab>
              <Tab value="duplicates">
                Duplicates
                {summaryStats.duplicates > 0 && (
                  <span className="ml-2 px-2 py-0.5 text-xs bg-amber-500/20 text-amber-400 rounded-full">
                    {summaryStats.duplicates}
                  </span>
                )}
              </Tab>
            </TabList>

            <TabPanels className="h-[calc(100%-3rem)]">
              <TabPanel value="summary" className="h-full overflow-auto p-6">
                <ValidationSummary stats={summaryStats} />
              </TabPanel>

              <TabPanel value="contradictions" className="h-full">
                <ContradictionList
                  selectedId={selectedContradictionId}
                  onSelect={setSelectedContradictionId}
                />
              </TabPanel>

              <TabPanel value="duplicates" className="h-full">
                <DuplicateList
                  selectedIds={selectedDuplicateIds}
                  onSelect={setSelectedDuplicateIds}
                  onMerge={handleMerge}
                />
              </TabPanel>
            </TabPanels>
          </Tabs>
        </div>
      </div>

      {/* Detail sidebar */}
      {(selectedContradictionId || selectedDuplicateIds) && (
        <div className="w-96 border-l border-darkroom-gray-700 bg-darkroom-card flex flex-col">
          <div className="p-4 border-b border-darkroom-gray-700 flex items-center justify-between">
            <h3 className="font-semibold text-darkroom-text-primary">
              {selectedContradictionId
                ? 'Contradiction Detail'
                : 'Duplicate Detail'}
            </h3>
            <button
              onClick={() => {
                setSelectedContradictionId(null);
                setSelectedDuplicateIds(null);
              }}
              className="text-darkroom-text-tertiary hover:text-darkroom-text-primary"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          <div className="flex-1 overflow-auto p-4">
            {selectedContradictionId ? (
              <ContradictionDetail id={selectedContradictionId} />
            ) : selectedDuplicateIds ? (
              <DuplicateDetail
                ids={selectedDuplicateIds}
                onMerge={() => setShowMergeModal(true)}
              />
            ) : null}
          </div>
        </div>
      )}

      {/* Merge Modal */}
      {showMergeModal && selectedDuplicateIds && (
        <MergeModal
          sourceId={selectedDuplicateIds[0]}
          targetId={selectedDuplicateIds[1]}
          onClose={() => setShowMergeModal(false)}
          onConfirm={() => {
            // Handle merge
            setShowMergeModal(false);
            setSelectedDuplicateIds(null);
          }}
        />
      )}
    </div>
  );
};

// Subcomponents

interface StatCardProps {
  label: string;
  value: string | number;
  icon: React.ReactNode;
  status?: 'success' | 'warning' | 'error';
}

const StatCard: React.FC<StatCardProps> = ({ label, value, icon, status }) => {
  const statusColors = {
    success: 'border-l-green-500',
    warning: 'border-l-amber-500',
    error: 'border-l-red-500',
  };

  return (
    <div
      className={`bg-darkroom-bg border border-darkroom-gray-700 border-l-4 ${
        status ? statusColors[status] : 'border-l-darkroom-gray-600'
      } rounded-lg p-3`}
    >
      <div className="flex items-center justify-between">
        <span className="text-xs text-darkroom-text-secondary uppercase tracking-wide">
          {label}
        </span>
        {icon}
      </div>
      <div className="text-2xl font-bold text-darkroom-text-primary mt-1">
        {value}
      </div>
    </div>
  );
};

const ContradictionDetail: React.FC<{ id: string }> = ({ id }) => (
  <div className="space-y-4">
    <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
      <div className="flex items-center gap-2 text-red-400 mb-2">
        <AlertTriangle className="w-4 h-4" />
        <span className="font-medium">Data Mismatch</span>
      </div>
      <p className="text-sm text-darkroom-text-secondary">
        Multiple sources provide conflicting information for this entity.
      </p>
    </div>

    <div>
      <h4 className="text-sm font-medium text-darkroom-text-primary mb-2">
        Conflicting Values
      </h4>
      <div className="space-y-2">
        <ConflictRow
          field="Location"
          values={['New York, NY', 'Brooklyn, NY']}
          sources={['Document A', 'Document B']}
        />
        <ConflictRow
          field="Role"
          values={['Engineer', 'Manager']}
          sources={['Meeting Notes', 'Wiki']}
        />
      </div>
    </div>

    <div className="pt-4 border-t border-darkroom-gray-700">
      <Button className="w-full" leftIcon={<CheckCircle className="w-4 h-4" />}>
        Resolve
      </Button>
    </div>
  </div>
);

const ConflictRow: React.FC<{
  field: string;
  values: string[];
  sources: string[];
}> = ({ field, values, sources }) => (
  <div className="p-3 bg-darkroom-bg rounded border border-darkroom-gray-700">
    <div className="text-sm font-medium text-darkroom-text-primary mb-2">
      {field}
    </div>
    {values.map((value, i) => (
      <div
        key={i}
        className="flex items-center justify-between py-1 text-sm"
      >
        <span className="text-darkroom-text-primary">{value}</span>
        <span className="text-xs text-darkroom-text-tertiary">
          from {sources[i]}
        </span>
      </div>
    ))}
  </div>
);

const DuplicateDetail: React.FC<{
  ids: [string, string];
  onMerge: () => void;
}> = ({ ids, onMerge }) => (
  <div className="space-y-4">
    <div className="p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg">
      <div className="flex items-center gap-2 text-amber-400 mb-2">
        <Copy className="w-4 h-4" />
        <span className="font-medium">Potential Duplicate</span>
      </div>
      <p className="text-sm text-darkroom-text-secondary">
        These entities appear to represent the same real-world object.
      </p>
    </div>

    <div className="grid grid-cols-2 gap-2">
      <div className="p-3 bg-darkroom-bg rounded border border-darkroom-gray-700">
        <div className="text-xs text-darkroom-text-tertiary mb-1">Entity A</div>
        <div className="text-sm font-medium text-darkroom-text-primary">
          John Smith
        </div>
        <div className="text-xs text-darkroom-text-secondary">ID: {ids[0]}</div>
      </div>
      <div className="p-3 bg-darkroom-bg rounded border border-darkroom-gray-700">
        <div className="text-xs text-darkroom-text-tertiary mb-1">Entity B</div>
        <div className="text-sm font-medium text-darkroom-text-primary">
          J. Smith
        </div>
        <div className="text-xs text-darkroom-text-secondary">ID: {ids[1]}</div>
      </div>
    </div>

    <div className="pt-4 border-t border-darkroom-gray-700 space-y-2">
      <Button className="w-full" leftIcon={<Merge className="w-4 h-4" />} onClick={onMerge}>
        Merge Entities
      </Button>
      <Button variant="ghost" className="w-full">
        Mark as Different
      </Button>
    </div>
  </div>
);

export default ValidationPage;
