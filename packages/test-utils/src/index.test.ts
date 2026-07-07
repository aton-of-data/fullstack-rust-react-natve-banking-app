import { describe, expect, it } from 'vitest';

import { createPageResponse, minorUnits, TEST_USER_ID } from './index.js';

describe('@ficus/test-utils', () => {
  it('creates paginated responses', () => {
    expect(createPageResponse([{ id: 1 }], 'cursor-2')).toEqual({
      items: [{ id: 1 }],
      next_cursor: 'cursor-2',
    });
  });

  it('exposes stable fixture constants', () => {
    expect(minorUnits(500)).toBe('500');
    expect(TEST_USER_ID).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i);
  });
});
