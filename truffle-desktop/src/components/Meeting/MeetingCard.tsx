import { formatDistanceToNow } from 'date-fns';
import { Calendar, Clock, Users, Play, Loader2, CheckCircle, AlertCircle } from 'lucide-react';
import type { Meeting } from '../../types/meeting';
import { MeetingType, ExtractionStatus } from '../../types/meeting';

export interface MeetingCardProps {
  meeting: Meeting;
  onClick?: () => void;
  showExtractButton?: boolean;
  onExtract?: () => void;
  isExtracting?: boolean;
}

const meetingTypeColors: Record<MeetingType, string> = {
  [MeetingType.Standup]: 'border-l-blue-500',
  [MeetingType.Planning]: 'border-l-green-500',
  [MeetingType.Review]: 'border-l-purple-500',
  [MeetingType.Retrospective]: 'border-l-orange-500',
  [MeetingType.OneOnOne]: 'border-l-pink-500',
  [MeetingType.Client]: 'border-l-yellow-500',
  [MeetingType.Team]: 'border-l-cyan-500',
  [MeetingType.Workshop]: 'border-l-indigo-500',
  [MeetingType.Other]: 'border-l-gray-500',
};

const meetingTypeLabels: Record<MeetingType, string> = {
  [MeetingType.Standup]: 'Standup',
  [MeetingType.Planning]: 'Planning',
  [MeetingType.Review]: 'Review',
  [MeetingType.Retrospective]: 'Retro',
  [MeetingType.OneOnOne]: '1:1',
  [MeetingType.Client]: 'Client',
  [MeetingType.Team]: 'Team',
  [MeetingType.Workshop]: 'Workshop',
  [MeetingType.Other]: 'Other',
};

function ExtractionStatusBadge({ status }: { status: ExtractionStatus }) {
  switch (status) {
    case ExtractionStatus.Completed:
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-green-500/10 text-green-400 text-xs">
          <CheckCircle size={12} />
          Extracted
        </span>
      );
    case ExtractionStatus.Extracting:
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 text-xs">
          <Loader2 size={12} className="animate-spin" />
          Extracting
        </span>
      );
    case ExtractionStatus.Failed:
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-red-500/10 text-red-400 text-xs">
          <AlertCircle size={12} />
          Failed
        </span>
      );
    default:
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-gray-500/10 text-gray-400 text-xs">
          Pending
        </span>
      );
  }
}

function ParticipantAvatars({ participants }: { participants: Meeting['participants'] }) {
  const displayParticipants = participants.slice(0, 3);
  const remainingCount = participants.length - 3;

  return (
    <div className="flex items-center">
      <div className="flex -space-x-2">
        {displayParticipants.map((participant, index) => (
          <div
            key={participant.id}
            className="w-7 h-7 rounded-full bg-gradient-to-br from-blue-500 to-purple-500 
                       border-2 border-gray-800 flex items-center justify-center text-xs font-medium text-white"
            style={{ zIndex: displayParticipants.length - index }}
            title={participant.name}
          >
            {participant.avatarUrl ? (
              <img src={participant.avatarUrl} alt={participant.name} className="w-full h-full rounded-full object-cover" />
            ) : (
              participant.name.charAt(0).toUpperCase()
            )}
          </div>
        ))}
      </div>
      {remainingCount > 0 && (
        <span className="ml-2 text-xs text-gray-500">+{remainingCount}</span>
      )}
    </div>
  );
}

export function MeetingCard({ 
  meeting, 
  onClick, 
  showExtractButton = false, 
  onExtract,
  isExtracting = false,
}: MeetingCardProps) {
  const borderColor = meetingTypeColors[meeting.meetingType] || meetingTypeColors[MeetingType.Other];
  const typeLabel = meetingTypeLabels[meeting.meetingType] || 'Other';
  const duration = meeting.duration || (meeting.endedAt 
    ? Math.round((new Date(meeting.endedAt).getTime() - new Date(meeting.startedAt).getTime()) / 60000)
    : null);

  return (
    <div
      onClick={onClick}
      className={`
        relative p-4 rounded-lg border border-gray-700 bg-gray-800/50 
        border-l-4 ${borderColor}
        transition-all duration-200 group
        ${onClick ? 'cursor-pointer hover:border-gray-500 hover:bg-gray-800 hover:shadow-lg' : ''}
      `}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-3 mb-2">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
              {typeLabel}
            </span>
            <ExtractionStatusBadge status={meeting.extractionStatus} />
          </div>
          <h3 className="font-semibold text-gray-100 truncate group-hover:text-blue-400 transition-colors">
            {meeting.title}
          </h3>
        </div>
        
        {showExtractButton && meeting.extractionStatus === ExtractionStatus.Pending && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onExtract?.();
            }}
            disabled={isExtracting}
            className="
              p-2 rounded-lg bg-blue-500/10 text-blue-400 
              hover:bg-blue-500 hover:text-white
              transition-colors opacity-0 group-hover:opacity-100
              disabled:opacity-50 disabled:cursor-not-allowed
            "
            title="Extract entities"
          >
            {isExtracting ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} />}
          </button>
        )}
      </div>

      {/* Meta info */}
      <div className="flex items-center gap-4 text-sm text-gray-400 mb-3">
        <div className="flex items-center gap-1.5">
          <Calendar size={14} />
          <span>{formatDistanceToNow(new Date(meeting.startedAt), { addSuffix: true })}</span>
        </div>
        {duration && (
          <div className="flex items-center gap-1.5">
            <Clock size={14} />
            <span>{duration} min</span>
          </div>
        )}
      </div>

      {/* Participants */}
      <div className="flex items-center justify-between">
        <ParticipantAvatars participants={meeting.participants} />
        <div className="flex items-center gap-1 text-xs text-gray-500">
          <Users size={12} />
          <span>{meeting.participants.length}</span>
        </div>
      </div>
    </div>
  );
}

export default MeetingCard;
