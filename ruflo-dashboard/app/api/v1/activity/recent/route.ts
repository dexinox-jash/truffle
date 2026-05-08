import { NextResponse } from 'next/server'
import { recentActivity } from '@/lib/mock-data'

export async function GET() {
  return NextResponse.json({ data: recentActivity })
}
