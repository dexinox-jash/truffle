export interface Meeting {
  id: string
  title: string
  scheduled_start_at: string
  status: 'scheduled' | 'completed' | 'cancelled'
  participants: Array<{
    id: string
    name: string
  }>
}

export interface DashboardStats {
  total_meetings: number
  this_month: number
  hours_recorded: number
  insights_generated: number
}

export interface ActivityItem {
  id: string
  type: 'meeting_completed' | 'transcript_generated' | 'insight_shared' | 'meeting_scheduled'
  title: string
  time: string
}

export const meetings: Meeting[] = [
  {
    id: 'm1',
    title: 'Q4 Roadmap Planning',
    scheduled_start_at: '2026-05-12T10:00:00Z',
    status: 'scheduled',
    participants: [
      { id: 'p1', name: 'Alex Chen' },
      { id: 'p2', name: 'Sarah Miller' },
      { id: 'p3', name: 'James Wilson' },
    ],
  },
  {
    id: 'm2',
    title: 'Product Design Review',
    scheduled_start_at: '2026-05-11T14:30:00Z',
    status: 'completed',
    participants: [
      { id: 'p4', name: 'Maria Garcia' },
      { id: 'p5', name: 'David Kim' },
    ],
  },
  {
    id: 'm3',
    title: 'Engineering Standup',
    scheduled_start_at: '2026-05-13T09:00:00Z',
    status: 'scheduled',
    participants: [
      { id: 'p1', name: 'Alex Chen' },
      { id: 'p6', name: 'Emily Zhang' },
      { id: 'p7', name: 'Ryan Patel' },
      { id: 'p8', name: 'Lisa Thompson' },
    ],
  },
  {
    id: 'm4',
    title: 'Customer Feedback Session',
    scheduled_start_at: '2026-05-10T16:00:00Z',
    status: 'completed',
    participants: [
      { id: 'p9', name: 'Michael Brown' },
      { id: 'p10', name: 'Jennifer Lee' },
    ],
  },
  {
    id: 'm5',
    title: 'AI Model Training Sync',
    scheduled_start_at: '2026-05-14T11:00:00Z',
    status: 'scheduled',
    participants: [
      { id: 'p11', name: 'Dr. Alan Turing' },
      { id: 'p12', name: 'Sophia Anderson' },
    ],
  },
  {
    id: 'm6',
    title: 'Security Audit Review',
    scheduled_start_at: '2026-05-09T13:00:00Z',
    status: 'cancelled',
    participants: [
      { id: 'p13', name: 'Robert Taylor' },
      { id: 'p14', name: 'Amanda White' },
    ],
  },
  {
    id: 'm7',
    title: 'Marketing Campaign Launch',
    scheduled_start_at: '2026-05-15T15:00:00Z',
    status: 'scheduled',
    participants: [
      { id: 'p15', name: 'Chris Martin' },
      { id: 'p16', name: 'Nina Patel' },
      { id: 'p17', name: 'Tom Bradley' },
    ],
  },
  {
    id: 'm8',
    title: 'Infrastructure Scaling Discussion',
    scheduled_start_at: '2026-05-08T10:00:00Z',
    status: 'completed',
    participants: [
      { id: 'p18', name: 'Kevin O\'Brien' },
      { id: 'p19', name: 'Rachel Green' },
    ],
  },
]

export const dashboardStats: DashboardStats = {
  total_meetings: 247,
  this_month: 34,
  hours_recorded: 186,
  insights_generated: 523,
}

export const recentActivity: ActivityItem[] = [
  {
    id: 'a1',
    type: 'meeting_completed',
    title: 'Product Design Review',
    time: '1h ago',
  },
  {
    id: 'a2',
    type: 'transcript_generated',
    title: 'Q4 Roadmap Planning transcript ready',
    time: '2h ago',
  },
  {
    id: 'a3',
    type: 'insight_shared',
    title: 'AI identified 3 action items from Engineering Standup',
    time: '3h ago',
  },
  {
    id: 'a4',
    type: 'meeting_scheduled',
    title: 'Security Audit Review scheduled for tomorrow',
    time: '5h ago',
  },
  {
    id: 'a5',
    type: 'meeting_completed',
    title: 'Customer Feedback Session',
    time: 'Yesterday',
  },
]
