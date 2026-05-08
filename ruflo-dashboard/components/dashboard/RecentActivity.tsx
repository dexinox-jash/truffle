export function RecentActivity() {
  return (
    <div className="rounded-lg bg-white shadow">
      <div className="p-6">
        <h3 className="text-base font-semibold leading-6 text-gray-900">
          Recent Activity
        </h3>
        <div className="mt-6 flow-root">
          <ul className="-mb-8">
            <li className="relative pb-8">
              <div className="relative flex space-x-3">
                <div>
                  <span className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-500 ring-8 ring-white">
                    <span className="text-white">M</span>
                  </span>
                </div>
                <div className="flex min-w-0 flex-1 justify-between space-x-4 pt-1.5">
                  <div>
                    <p className="text-sm text-gray-500">
                      Meeting <span className="font-medium text-gray-900">Team Standup</span> completed
                    </p>
                  </div>
                  <div className="whitespace-nowrap text-right text-sm text-gray-500">
                    1h ago
                  </div>
                </div>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  )
}
