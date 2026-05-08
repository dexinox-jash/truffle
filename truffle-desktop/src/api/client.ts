import { invoke } from '@tauri-apps/api/tauri';

// Generic invoke wrapper with error handling
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    console.error(`Command ${command} failed:`, error);
    throw error;
  }
}

// API error type
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}

// Pagination params
export interface PaginationParams {
  limit?: number;
  offset?: number;
}

// Paginated response
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
  hasMore: boolean;
}
