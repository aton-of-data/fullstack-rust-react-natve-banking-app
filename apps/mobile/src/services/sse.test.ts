import { describe, expect, it, vi, afterEach } from 'vitest';

import { parseSseBuffer, parseSseChunk, subscribeFeedSse } from './sse';

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

describe('parseSseBuffer', () => {
  it('keeps incomplete trailing lines as remainder', () => {
    const first = parseSseBuffer(
      'data: {"transfer_id":"t1","sender_username":"alice","recipient_username":"bob","amount_minor":"100","currency":"USD","created_at":"2025-01-01T00:00:00Z"}\n\ndata: {"transfer_id":"t2"',
    );
    expect(first.items).toHaveLength(1);
    expect(first.remainder).toContain('transfer_id":"t2"');

    const second = parseSseBuffer(
      `${first.remainder},"sender_username":"bob","recipient_username":"charlie","amount_minor":"200","currency":"USD","created_at":"2025-01-01T00:00:01Z"}\n\n`,
    );
    expect(second.items).toHaveLength(1);
    expect(second.items[0]?.transfer_id).toBe('t2');
  });

  it('captures last event id', () => {
    const parsed = parseSseBuffer(
      'id: evt-9\ndata: {"transfer_id":"t9","sender_username":"a","recipient_username":"b","amount_minor":"1","currency":"USD","created_at":"2025-01-01T00:00:00Z"}\n\n',
    );
    expect(parsed.lastEventId).toBe('evt-9');
  });
});

describe('subscribeFeedSse', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('streams parsed feed items and sends Last-Event-ID when provided', async () => {
    class MockXHR {
      static last: MockXHR | null = null;
      open = vi.fn();
      setRequestHeader = vi.fn();
      send = vi.fn();
      abort = vi.fn();
      responseText = '';
      onprogress: (() => void) | null = null;
      onerror: (() => void) | null = null;
      onabort: (() => void) | null = null;
      onload: (() => void) | null = null;

      constructor() {
        MockXHR.last = this;
      }
    }

    vi.stubGlobal('XMLHttpRequest', MockXHR as unknown as typeof XMLHttpRequest);

    const onItem = vi.fn();
    const onEventId = vi.fn();
    const controller = new AbortController();
    const promise = subscribeFeedSse({
      url: 'http://localhost/v1/feed/stream',
      token: 'token',
      onItem,
      signal: controller.signal,
      lastEventId: 'evt-1',
      onEventId,
    });

    const xhr = MockXHR.last;
    expect(xhr).toBeTruthy();
    expect(xhr?.open).toHaveBeenCalledWith('GET', 'http://localhost/v1/feed/stream');
    expect(xhr?.setRequestHeader).toHaveBeenCalledWith('Authorization', 'Bearer token');
    expect(xhr?.setRequestHeader).toHaveBeenCalledWith('Last-Event-ID', 'evt-1');

    xhr!.responseText =
      'id: evt-2\ndata: {"transfer_id":"t1","sender_username":"alice","recipient_username":"bob","amount_minor":"100","currency":"USD","created_at":"2025-01-01T00:00:00Z"}\n\n';
    xhr!.onprogress?.();

    expect(onItem).toHaveBeenCalledWith(
      expect.objectContaining({ transfer_id: 't1', sender_username: 'alice' }),
    );
    expect(onEventId).toHaveBeenCalledWith('evt-2');

    xhr!.onerror?.();
    await expect(promise).rejects.toThrow('SSE connection failed');
  });
});
