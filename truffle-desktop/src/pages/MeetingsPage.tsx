import React, { useEffect, useCallback, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Calendar,
  Search,
  Filter,
  Download,
  Mic,
  FileText,
  CheckCircle,
  Clock,
  X,
  ChevronRight,
} from 'lucide-react';
import { useEntityStore } from '../store/entityStore';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { MeetingList } from '../components/Meeting/MeetingList';
import { MeetingDetail } from '../components/Meeting/MeetingDetail';
import { ExtractionPanel } from '../components/Meeting/ExtractionPanel';
import { EmptyState } from '../components/ui/EmptyState';

export const MeetingsPage: React.FC = () => {
  const { id } = useParams<{ id?: string }>();
  const navigate = useNavigate();

  const {
    selectedMeetingId,
    setSelectedMeetingId,
    meetingSearchQuery,
    setMeetingSearchQuery,
  } = useEntityStore();

  const [dateRange, setDateRange] = useState<{
    from: Date | null;
    to: Date | null;
  }>({ from: null, to: null });
  const [hasExtractionFilter, setHasExtractionFilter] = useState<boolean |
    null>(null);
  const [showExtractionPanel, setShowExtractionPanel] = useState(false);

  // Sync URL params with store
  useEffect(() => {
    if (id) {
      setSelectedMeetingId(id);
    }
  }, [id, setSelectedMeetingId]);

  const handleSelectMeeting = useCallback(
    (meetingId: string) => {
      setSelectedMeetingId(meetingId);
      navigate(`/meetings/${meetingId}`);
    },
    [setSelectedMeetingId, navigate]
  );

  const handleCloseDetail = useCallback(() => {
    setSelectedMeetingId(null);
    navigate('/meetings');
  }, [setSelectedMeetingId, navigate]);

  const filter = {
    search: meetingSearchQuery,
    dateRange,
    hasExtraction: hasExtractionFilter,
  };

  return (
    <div className="h-[calc(100vh-4rem)] flex bg-darkroom-bg">
      {/* Sidebar - Meeting List */}
      <div className="w-96 border-r border-darkroom-gray-700 flex flex-col bg-darkroom-card">
        {/* Header */}
        <div className="p-4 border-b border-darkroom-gray-700">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-darkroom-text-primary flex items-center gap-2">
              <Calendar className="w-5 h-5" />
              Meetings
            </h2>
            <Button size="sm" leftIcon={<Mic className="w-4 h-4" />}>
              Record
            </Button>
          </div>

          {/* Search */}
          <div className="relative mb-3">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-darkroom-text-tertiary" />
            <Input
              placeholder="Search meetings..."
              value={meetingSearchQuery}
              onChange={(e) => setMeetingSearchQuery(e.target.value)}
              className="pl-9 pr-9"
            />
            {meetingSearchQuery && (
              <button
                onClick={() => setMeetingSearchQuery('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-darkroom-text-tertiary hover:text-darkroom-text-primary"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>

          {/* Filters */}
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              leftIcon={<Filter className="w-3.5 h-3.5" />}
              className={dateRange.from ? 'text-accent' : ''}
            >
              Date
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() =>
                setHasExtractionFilter(
                  hasExtractionFilter === true
                    ? null
                    : true
                )
              }
              className={hasExtractionFilter ? 'text-accent' : ''}
            >
              Has Extraction
            </Button>
          </div>
        </div>

        {/* Meeting List */}
        <div className="flex-1 overflow-hidden">
          <MeetingList
            filter={filter}
            selectedId={selectedMeetingId}
            onSelect={handleSelectMeeting}
          />
        </div>

        {/* Footer */}
        <div className="p-3 border-t border-darkroom-gray-700">
          <Button
            variant="ghost"
            size="sm"
            className="w-full"
            leftIcon={<Download className="w-4 h-4" />}
          >
            Export All
          </Button>
        </div>
      </div>

      {/* Main content - Meeting Detail */}
      <div className="flex-1 overflow-auto">
        {selectedMeetingId ? (
          <MeetingDetail
            meetingId={selectedMeetingId}
            onClose={handleCloseDetail}
            onViewExtraction={() => setShowExtractionPanel(true)}
          />
        ) : (
          <EmptyState
            icon={<Calendar className="w-16 h-16" />}
            title="No Meeting Selected"
            description="Select a meeting from the list to view details and extractions."
            action={
              <div className="flex gap-2">
                <Button variant="secondary" leftIcon={<Mic className="w-4 h-4" />}>
                  Start Recording
                </Button>
                <Button variant="ghost" leftIcon={<FileText className="w-4 h-4" />}>
                  Upload Transcript
                </Button>
              </div>
            }
          />
        )}
      </div>

      {/* Extraction Side Panel */}
      {showExtractionPanel && selectedMeetingId && (
        <div className="w-[480px] border-l border-darkroom-gray-700 bg-darkroom-card flex flex-col">
          <div className="p-4 border-b border-darkroom-gray-700 flex items-center justify-between">
            <h3 className="font-semibold text-darkroom-text-primary flex items-center gap-2">
              <CheckCircle className="w-5 h-5 text-green-500" />
              AI Extraction
            </h3>
            <button
              onClick={() => setShowExtractionPanel(false)}
              className="text-darkroom-text-tertiary hover:text-darkroom-text-primary"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          <div className="flex-1 overflow-auto">
            <ExtractionPanel meetingId={selectedMeetingId} />
          </div>

          <div className="p-4 border-t border-darkroom-gray-700">
            <div className="flex items-center justify-between text-sm text-darkroom-text-secondary">
              <span>Last extracted: 2 hours ago</span>
              <Button size="sm" variant="secondary">
                Re-extract
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default MeetingsPage;
