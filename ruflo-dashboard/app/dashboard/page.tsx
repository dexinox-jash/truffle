'use client'

import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { MeetingList } from '@/components/meetings/MeetingList'
import { StatsCard } from '@/components/dashboard/StatsCard'
import { RecentActivity } from '@/components/dashboard/RecentActivity'

export default function DashboardPage() {
  const { data: stats } = useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: async () => {
      const response = await api.get('/api/v1/analytics/meetings')
      return response.data
    },
  })

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow">
        <div className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">
            Dashboard
          </h1>
        </div>
      </header>

      <main className="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
          <StatsCard title="Total Meetings" value={stats?.total_meetings || 0} />
          <StatsCard title="This Month" value={stats?.this_month || 0} />
          <StatsCard title="Hours Recorded" value={stats?.hours_recorded || 0} />
          <StatsCard title="AI Insights" value={stats?.insights_generated || 0} />
        </div>

        <div className="mt-8 grid grid-cols-1 gap-8 lg:grid-cols-3">
          <div className="lg:col-span-2">
            <MeetingList />
          </div>
          <div>
            <RecentActivity />
          </div>
        </div>
      </main>
    </div>
  )
}
