import { NextRequest, NextResponse } from 'next/server'
import { meetings } from '@/lib/mock-data'

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  const limit = parseInt(searchParams.get('limit') || '10', 10)

  const data = meetings.slice(0, limit)

  return NextResponse.json({ data })
}
