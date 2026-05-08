import { useState, useMemo } from 'react';
import { format, isToday, isYesterday, startOfWeek, isSameWeek, parseISO } from 'date-fns';
import { Search, Upload, Calendar, Filter, Loader2 } from 'lucide-react';
import { MeetingCard } from './MeetingCard';
import { useMeetings, useExtractFromMeeting } from '../../hooks/useMeetings';
import type { MeetingFilter } from '../../types/meeting';
import type { Meeting } from '../../types/meeting';

export interface MeetingListProps {
  filter?: MeetingFilter;
  onSelect?: (meeting: Meeting) => void;
  onExtract?: (meetingId: string) => void;
  className?: string;
}

interface GroupedMeetings {
  today: Meeting[];
  yesterday: Meeting[];
  thisWeek: Meeting[];
  earlier: Meeting[];
}

function groupMeetings(meetings: Meeting[]): GroupedMeetings {
  const grouped: GroupedMeetings = {
    today: [],
    yesterday: [],
    thisWeek: [],
    earlier: [],
  };

  const now = new Date();
  const weekStart = startOfWeek(now, { weekStartsOn: 1 });

  meetings.forEach((meeting) => {
    const date = parseISO(meeting.startedAt);

    if (isToday(date)) {
      grouped.today.push(meeting);
    } else if (isYesterday(date)) {
      grouped.yesterday.push(meeting);
    } else if (isSameWeek(date, now, { weekStartsOn: 1 })) {
      grouped.thisWeek.push(meeting);
    } else {
      grouped.earlier.push(meeting);
    }
  });

  // Sort each group by date (newest first)
  const sortByDate = (a: Meeting, b: Meeting) => 
    new Date(b.startedAt).getTime() - new Date(a.startedAt).getTime();
  
  grouped.today.sort(sortByDate);
  grouped.yesterday.sort(sortByDate);
  grouped.thisWeek.sort(sortByDate);
  grouped.earlier.sort(sortByDate);

  return grouped;
}

function MeetingGroup({ 
  title, 
  meetings, 
  onSelect, 
  onExtract,
  extractingId,
}: { 
  title: string; 
  meetings: Meeting[]; 
  onSelect?: (meeting: Meeting) => void;
  onExtract?: (meetingId: string) => void;
  extractingId: string | null;
}) {
  if (meetings.length === 0) return null;

  return (
    <div className="mb-6">
      <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wide mb-3 px-1">
        {title}
      </h3>
      <div className="space-y-3">
        {meetings.map((meeting) => (
          <MeetingCard
            key={meeting.id}
            meeting={meeting}
            onClick={() => onSelect?.(meeting)}
            showExtractButton={true}
            onExtract={() => onExtract?.(meeting.id)}
            isExtracting={extractingId === meeting.id}
          />
        ))}
      </div>
    </div>
  );
}

export function MeetingList({ filter = {}, onSelect, onExtract, className = '' }: MeetingListProps) {
  const [searchQuery, setSearchQuery] = useState(filter.search || '');
  const [extractingId, setExtractingId] = useState<string | null>(null);
  
  const { data: meetings = [], isLoading, error } = useMeetings({
    ...filter,
    search: searchQuery,
  });

  const extractMutation = useExtractFromMeeting();

  const filteredMeetings = useMemo(() => {
    if (!searchQuery.trim()) return meetings;
    
    const query = searchQuery.toLowerCase();
    return meetings.filter((meeting) => 
      meeting.title.toLowerCase().includes(query) ||
      meeting.participants.some((p) => 
        p.name.toLowerCase().includes(query) || 
        p.email?.toLowerCase().includes(query)
      )
    );
  }, [meetings, searchQuery]);

  const grouped = useMemo(() => groupMeetings(filteredMeetings), [filteredMeetings]);

  const handleExtract = async (meetingId: string) => {
    setExtractingId(meetingId);
    try {
      await extractMutation.mutateAsync(meetingId);
      onExtract?.(meetingId);
    } finally {
      setExtractingId(null);
    }
  };

  if (isLoading) {
    return (
      <div className={`flex items-center justify-center h-64 ${className}`}>
        <Loader2 size={32} className="animate-spin text-blue-500" />
      </div>
    );
  }

  if (error) {
    return (
      <div className={`flex flex-col items-center justify-center h-64 text-gray-400 ${className}`}>
        <AlertCircle size={32} className="mb-2 text-red-500" />
        <p>Failed to load meetings</p>
        <p className="text-sm text-gray-500">{error.message}</p>
      </div>
    );
  }

  const hasNoMeetings = 
    grouped.today.length === 0 && 
    grouped.yesterday.length === 0 && 
    grouped.thisWeek.length === 0 && 
    grouped.earlier.length === 0;

  return (
    <div className={className}>
      {/* Header */}
      <div className="flex items-center justify-between gap-4 mb-6">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={18} />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search meetings, participants..."
            className="w-full pl-10 pr-4 py-2.5 bg-gray-800/50 border border-gray-700 rounded-lg 
                       text-gray-200 placeholder-gray-500
                       focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50
                       transition-all"
          />
        </div>
        <button
          className="flex items-center gap-2 px-4 py-2.5 bg-blue-500/10 text-blue-400 
                     border border-blue-500/20 rounded-lg
                     hover:bg-blue-500 hover:text-white hover:border-blue-500
                     transition-all"
        >
          <Upload size={18} />
          <span>Import Meeting</span>
        </button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-2 mb-6">
        <button className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-400 
                           border border-gray-700 rounded-lg hover:border-gray-500 hover:text-gray-300">
          <Calendar size={14} />
          <span>All Time</span>
        </button>
        <button className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-400 
                           border border-gray-700 rounded-lg hover:border-gray-500 hover:text-gray-300">
          <Filter size={14} />
          <span>Filter</span>
        </button>
      </div>

      {/* Meeting Groups */}
      {hasNoMeetings ? (
        <div className="flex flex-col items-center justify-center h-64 text-gray-400">
          <Calendar size={48} className="mb-4 text-gray-600" />
          <p className="text-lg font-medium text-gray-300 mb-1">No meetings found</p>
          <p className="text-sm text-gray-500 mb-4">
            {searchQuery ? 'Try adjusting your search' : 'Get started by importing your first meeting'}
          </p>
          {!searchQuery && (
            <button
              className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg
                         hover:bg-blue-600 transition-colors"
            >
              <Upload size={18} />
              <span>Import Meeting</span>
            </button>
          )}
        </div>
      ) : (
        <>
          <MeetingGroup
            title="Today"
            meetings={grouped.today}
            onSelect={onSelect}
            onExtract={handleExtract}
            extractingId={extractingId}
          />
          <MeetingGroup
            title="Yesterday"
            meetings={grouped.yesterday}
            onSelect={onSelect}
            onExtract={handleExtract}
            extractingId={extractingId}
          />
          <MeetingGroup
            title="This Week"
            meetings={grouped.thisWeek}
            onSelect={onSelect}
            onExtract={handleExtract}
            extractingId={extractingId}
          />
          <MeetingGroup
            title="Earlier"
            meetings={grouped.earlier}
            onSelect={onSelect}
            onExtract={handleExtract}
            extractingId={extractingId}
          />
        </>
      )}
    </div>
  );
}

// Need to import for error state
import { AlertCircle } from 'lucide-react';

export default MeetingList;
