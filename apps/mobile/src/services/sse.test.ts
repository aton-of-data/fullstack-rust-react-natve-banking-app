import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';

import { parseSseChunk, subscribeFeedSse } from './sse';

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

describe('subscribeFeedSse', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('streams parsed feed items and resolves on abort', async () => {
    class MockXHR {
      static last: MockXHR | null = null;
      open = vi.fn();
      setRequestHeader = vi.fn();
      send = vi.fn();
      abort = vi.fn();
      responseText = '';
      onprogress: (() => void) | null = null;
      onabort: (() => void) | null = null;

      constructor() {
        MockXHR.last = this;
      }
    }

    vi.stubGlobal('XMLHttpRequest', MockXHR as unknown as typeof XMLHttpRequest);

    const onItem = vi.fn();
    const controller = new AbortController();
    const promise = subscribeFeedSse(
      'http://localhost/v1/feed/stream',
      'token',
      onItem,
      controller.signal,
    );

    const xhr = MockXHR.last;
    expect(xhr).toBeTruthy();
    expect(xhr?.open).toHaveBeenCalledWith('GET', 'http://localhost/v1/feed/stream');
    expect(xhr?.setRequestHeader).toHaveBeenCalledWith('Authorization', 'Bearer token');

    xhr!.responseText =
      'data: {"transfer_id":"t1","sender_username":"alice","recipient_username":"bob","amount_minor":"100","currency":"USD","created_at":"2025-01-01T00:00:00Z"}\n';
    xhr!.onprogress?.();

    expect(onItem).toHaveBeenCalledWith(
      expect.objectContaining({ transfer_id: 't1', sender_username: 'alice' }),
    );

    xhr!.onerror?.();
    await expect(promise).rejects.toThrow('SSE connection failed');
  });
});
