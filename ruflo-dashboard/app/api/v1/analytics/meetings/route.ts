import { NextResponse } from 'next/server'
import { dashboardStats } from '@/lib/mock-data'

export async function GET() {
  return NextResponse.json(dashboardStats)
}
