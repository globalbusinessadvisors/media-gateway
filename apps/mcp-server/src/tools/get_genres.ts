/**
 * MCP Tool: get_genres
 * Get list of available content genres
 */

import { z } from 'zod';
import { MCPTool, Genre } from '../types/index.js';
import { config } from '../config.js';

export const getGenresSchema = z.object({
  mediaType: z.enum(['movie', 'tv', 'all']).default('all').describe('Media type filter'),
});

export type GetGenresInput = z.infer<typeof getGenresSchema>;

export const getGenresTool: MCPTool = {
  name: 'get_genres',
  description: 'Get list of available content genres with descriptions',
  inputSchema: {
    type: 'object',
    properties: {
      mediaType: {
        type: 'string',
        enum: ['movie', 'tv', 'all'],
        default: 'all',
        description: 'Filter by media type',
      },
    },
  },
};

export async function executeGetGenres(
  input: GetGenresInput
): Promise<{ genres: Genre[] }> {
  try {
    const params = new URLSearchParams({
      mediaType: input.mediaType,
    });

    const response = await fetch(`${config.services.content}/api/v1/genres?${params}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      throw new Error(`Content service returned ${response.status}`);
    }

    const data = await response.json();
    return { genres: data.genres || [] };
  } catch (error) {
    console.error('Get genres error:', error);
    throw new Error(`Failed to get genres: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}
