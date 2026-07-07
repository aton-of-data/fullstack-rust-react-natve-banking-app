import type { PageResponse } from '@ficus/contracts';

/**
 * Builds a cursor-paginated API page for tests.
 *
 * @typeParam T Item type contained in the page.
 * @param items Page items.
 * @param nextCursor Optional next-page cursor.
 * @returns API-shaped page response.
 */
export function createPageResponse<T>(items: T[], nextCursor?: string | null): PageResponse<T> {
  if (nextCursor === undefined) {
    return { items };
  }
  return { items, next_cursor: nextCursor };
}

/**
 * Returns a canonical minor-unit string for test fixtures.
 *
 * @param value Integer minor units.
 * @returns Wire-format minor-unit string.
 */
export function minorUnits(value: string | number | bigint): string {
  if (typeof value === 'bigint') {
    return value.toString(10);
  }
  if (typeof value === 'number') {
    if (!Number.isInteger(value) || value < 0) {
      throw new Error('minorUnits fixture must be a non-negative integer');
    }
    return String(value);
  }
  return value;
}

/**
 * Fixed UUID useful for deterministic tests.
 */
export const TEST_USER_ID = '00000000-0000-4000-8000-000000000001';

/**
 * Fixed transfer UUID useful for deterministic tests.
 */
export const TEST_TRANSFER_ID = '00000000-0000-4000-8000-000000000002';

/**
 * Fixed ISO timestamp useful for deterministic tests.
 */
export const TEST_ISO_TIMESTAMP = '2026-01-01T00:00:00.000Z';

/**
 * Resolves on the next microtask. Prefer in unit tests over arbitrary `setTimeout`.
 *
 * @returns Promise that settles after the current call stack clears.
 */
export function flushPromises(): Promise<void> {
  return Promise.resolve();
}
