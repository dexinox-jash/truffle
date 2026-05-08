'use client'

import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { format } from 'date-fns'

interface Meeting {
  id: string
  title: string
  scheduled_start_at: string
  status: string
  participants: Array<{
    id: string
    name: string
  }>
}

export function MeetingList() {
  const { data: meetings, isLoading } = useQuery({
    queryKey: ['meetings'],
    queryFn: async () => {
      const response = await api.get('/api/v1/meetings?limit=10')
      return response.data.data as Meeting[]
    },
  })

  if (isLoading) {
    return <div className="text-center py-10">Loading...</div>
  }

  return (
    <div className="rounded-lg bg-white shadow">
      <div className="p-6">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold leading-6 text-gray-900">
            Upcoming Meetings
          </h3>
          <a
            href="/meetings"
            className="text-sm font-semibold text-indigo-600 hover:text-indigo-500"
          >
            View all
          </a>
        </div>

        <div className="mt-6 flow-root">
          <ul className="-my-5 divide-y divide-gray-200">
            {meetings?.map((meeting) => (
              <li key={meeting.id} className="py-4">
                <div className="flex items-center space-x-4">
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium text-gray-900">
                      {meeting.title}
                    </p>
                    <p className="text-sm text-gray-500">
                      {format(new Date(meeting.scheduled_start_at), 'MMM d, yyyy h:mm a')}
                    </p>
                  </div>
                  <div>
                    <span
                      className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${
                        meeting.status === 'scheduled'
                          ? 'bg-blue-50 text-blue-700'
                          : 'bg-green-50 text-green-700'
                      }`}
                    >
                      {meeting.status}
                    </span>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}
