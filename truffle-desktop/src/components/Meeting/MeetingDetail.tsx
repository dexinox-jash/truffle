import { useState } from 'react';
import { format } from 'date-fns';
import { 
  X, 
  Calendar, 
  Clock, 
  Users, 
  Play, 
  FileText, 
  Sparkles, 
  Network,
  Save,
  Edit2,
  Loader2,
  CheckCircle,
} from 'lucide-react';
import { useMeeting, useExtractFromMeeting, useUpdateMeeting } from '../../hooks/useMeetings';
import { useExtractionResult } from '../../hooks/useMeetings';
import { ExtractedEntities } from './ExtractedEntities';
import { ExtractionStatus } from '../../types/meeting';

export interface MeetingDetailProps {
  meetingId: string;
  onClose?: () => void;
  onExtract?: () => void;
}

type TabType = 'transcript' | 'summary' | 'entities';

function ParticipantList({ participants }: { participants: Array<{ id: string; name: string; email?: string; avatarUrl?: string }> }) {
  return (
    <div className="flex flex-wrap gap-2">
      {participants.map((participant) => (
        <div
          key={participant.id}
          className="flex items-center gap-2 px-3 py-1.5 bg-gray-800/50 border border-gray-700 rounded-full"
        >
          <div className="w-6 h-6 rounded-full bg-gradient-to-br from-blue-500 to-purple-500 
                          flex items-center justify-center text-xs font-medium text-white">
            {participant.avatarUrl ? (
              <img 
                src={participant.avatarUrl} 
                alt={participant.name} 
                className="w-full h-full rounded-full object-cover" 
              />
            ) : (
              participant.name.charAt(0).toUpperCase()
            )}
          </div>
          <span className="text-sm text-gray-300">{participant.name}</span>
          {participant.isExternal && (
            <span className="text-xs text-gray-500">(External)</span>
          )}
        </div>
      ))}
    </div>
  );
}

export function MeetingDetail({ meetingId, onClose, onExtract }: MeetingDetailProps) {
  const [activeTab, setActiveTab] = useState<TabType>('transcript');
  const [isEditingSummary, setIsEditingSummary] = useState(false);
  const [editedSummary, setEditedSummary] = useState('');

  const { data: meeting, isLoading: meetingLoading } = useMeeting(meetingId);
  const { data: extractionResult } = useExtractionResult(meetingId);
  const extractMutation = useExtractFromMeeting();
  const updateMutation = useUpdateMeeting();

  const isExtracting = meeting?.extractionStatus === ExtractionStatus.Extracting;
  const hasExtraction = meeting?.extractionStatus === ExtractionStatus.Completed;

  const handleExtract = async () => {
    await extractMutation.mutateAsync(meetingId);
    onExtract?.();
  };

  const handleSaveSummary = async () => {
    await updateMutation.mutateAsync({
      id: meetingId,
      input: { summary: editedSummary },
    });
    setIsEditingSummary(false);
  };

  const startEditingSummary = () => {
    setEditedSummary(meeting?.summary || '');
    setIsEditingSummary(true);
  };

  if (meetingLoading || !meeting) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 size={32} className="animate-spin text-blue-500" />
      </div>
    );
  }

  const duration = meeting.duration || (meeting.endedAt
    ? Math.round((new Date(meeting.endedAt).getTime() - new Date(meeting.startedAt).getTime()) / 60000)
    : null);

  const formattedDate = format(new Date(meeting.startedAt), 'MMMM d, yyyy h:mm a');

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Header */}
      <div className="flex items-start justify-between p-6 border-b border-gray-800">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2">
            <h2 className="text-xl font-semibold text-gray-100 truncate">{meeting.title}</h2>
            {hasExtraction && (
              <CheckCircle size={18} className="text-green-500" title="Entities extracted" />
            )}
          </div>
          
          <div className="flex items-center gap-4 text-sm text-gray-400 mb-4">
            <div className="flex items-center gap-1.5">
              <Calendar size={14} />
              <span>{formattedDate}</span>
            </div>
            {duration && (
              <div className="flex items-center gap-1.5">
                <Clock size={14} />
                <span>{duration} minutes</span>
              </div>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Users size={16} className="text-gray-500" />
            <ParticipantList participants={meeting.participants} />
          </div>
        </div>

        <div className="flex items-center gap-2 ml-4">
          {!hasExtraction && !isExtracting && (
            <button
              onClick={handleExtract}
              disabled={extractMutation.isPending}
              className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg
                         hover:bg-blue-600 transition-colors disabled:opacity-50"
            >
              {extractMutation.isPending ? (
                <Loader2 size={16} className="animate-spin" />
              ) : (
                <Play size={16} />
              )}
              <span>Extract Entities</span>
            </button>
          )}
          {onClose && (
            <button
              onClick={onClose}
              className="p-2 text-gray-400 hover:text-gray-200 hover:bg-gray-800 rounded-lg transition-colors"
            >
              <X size={20} />
            </button>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="flex items-center gap-1 px-6 border-b border-gray-800">
        <button
          onClick={() => setActiveTab('transcript')}
          className={`
            flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors
            ${activeTab === 'transcript'
              ? 'border-blue-500 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-gray-200'
            }
          `}
        >
          <FileText size={16} />
          Transcript
        </button>
        <button
          onClick={() => setActiveTab('summary')}
          className={`
            flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors
            ${activeTab === 'summary'
              ? 'border-blue-500 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-gray-200'
            }
          `}
        >
          <Sparkles size={16} />
          Summary
        </button>
        <button
          onClick={() => setActiveTab('entities')}
          className={`
            flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors
            ${activeTab === 'entities'
              ? 'border-blue-500 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-gray-200'
            }
          `}
        >
          <Network size={16} />
          Entities
          {extractionResult?.entities && (
            <span className="ml-1 px-1.5 py-0.5 text-xs bg-gray-700 rounded-full">
              {extractionResult.entities.length}
            </span>
          )}
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {activeTab === 'transcript' && (
          <div className="prose prose-invert max-w-none">
            {meeting.transcript ? (
              <pre className="font-mono text-sm text-gray-300 whitespace-pre-wrap bg-gray-800/50 
                             p-4 rounded-lg border border-gray-700">
                {meeting.transcript}
              </pre>
            ) : (
              <div className="text-center text-gray-500 py-12">
                <FileText size={48} className="mx-auto mb-4 text-gray-600" />
                <p>No transcript available for this meeting.</p>
              </div>
            )}
          </div>
        )}

        {activeTab === 'summary' && (
          <div className="max-w-3xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-medium text-gray-500 uppercase tracking-wide">AI-Generated Summary</h3>
              {isEditingSummary ? (
                <button
                  onClick={handleSaveSummary}
                  disabled={updateMutation.isPending}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-green-500/10 text-green-400 
                             rounded-lg hover:bg-green-500/20 transition-colors"
                >
                  <Save size={14} />
                  Save
                </button>
              ) : (
                <button
                  onClick={startEditingSummary}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-400 
                             rounded-lg hover:bg-gray-800 transition-colors"
                >
                  <Edit2 size={14} />
                  Edit
                </button>
              )}
            </div>

            {isEditingSummary ? (
              <textarea
                value={editedSummary}
                onChange={(e) => setEditedSummary(e.target.value)}
                className="w-full h-64 p-4 bg-gray-800/50 border border-gray-700 rounded-lg 
                           text-gray-200 resize-none focus:outline-none focus:border-blue-500
                           transition-all"
                placeholder="Enter meeting summary..."
              />
            ) : meeting.summary ? (
              <div className="prose prose-invert max-w-none">
                <div className="text-gray-300 leading-relaxed whitespace-pre-wrap">
                  {meeting.summary}
                </div>
              </div>
            ) : (
              <div className="text-center text-gray-500 py-12">
                <Sparkles size={48} className="mx-auto mb-4 text-gray-600" />
                <p className="mb-2">No summary available.</p>
                <p className="text-sm">Extract entities to generate an AI summary.</p>
              </div>
            )}
          </div>
        )}

        {activeTab === 'entities' && (
          <div>
            {!hasExtraction ? (
              <div className="text-center text-gray-500 py-12">
                <Network size={48} className="mx-auto mb-4 text-gray-600" />
                <p className="mb-2">No entities extracted yet.</p>
                <button
                  onClick={handleExtract}
                  disabled={extractMutation.isPending || isExtracting}
                  className="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg
                             hover:bg-blue-600 transition-colors disabled:opacity-50"
                >
                  {extractMutation.isPending || isExtracting ? 'Extracting...' : 'Extract Now'}
                </button>
              </div>
            ) : extractionResult ? (
              <ExtractedEntities
                meetingId={meetingId}
                entities={extractionResult.entities}
                relationships={extractionResult.relationships}
              />
            ) : (
              <div className="flex items-center justify-center py-12">
                <Loader2 size={24} className="animate-spin text-blue-500" />
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default MeetingDetail;
