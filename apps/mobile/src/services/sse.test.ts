import { describe, expect, it } from 'vitest';

import { parseSseChunk } from './sse';

describe('parseSseChunk', () => {
  it('parses feed items from data lines', () => {
    const chunk = [
      'data: {"transfer_id":"t1","sender_username":"alice","recipient_username":"bob","amount_minor":"100","currency":"USD","created_at":"2025-01-01T00:00:00Z"}',
      '',
    ].join('\n');
    const items = parseSseChunk(chunk);
    expect(items).toHaveLength(1);
    expect(items[0]?.transfer_id).toBe('t1');
  });

  it('ignores keep-alive events', () => {
    const items = parseSseChunk('data: keep-alive\n\n');
    expect(items).toHaveLength(0);
  });

  it('ignores duplicate malformed events safely', () => {
    const items = parseSseChunk('data: not-json\n');
    expect(items).toHaveLength(0);
  });
});
