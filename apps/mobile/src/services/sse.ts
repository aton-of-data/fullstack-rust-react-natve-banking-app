import type { FeedItem } from '@ficus/contracts';

/**
 * Result of parsing an SSE text buffer, including incomplete trailing data.
 */
export interface ParseSseBufferResult {
  /** Fully parsed feed items from complete events. */
  items: FeedItem[];
  /** Remainder of the buffer after the last complete line/event boundary. */
  remainder: string;
  /** Last SSE `id:` value seen, if any. */
  lastEventId: string | null;
}

/**
 * Parses a Server-Sent Events text buffer into feed items.
 * Incomplete trailing lines are returned as `remainder` for the next chunk.
 *
 * @param buffer Accumulated SSE text (previous remainder + new bytes).
 * @returns Parsed items, remainder, and last event id.
 */
export function parseSseBuffer(buffer: string): ParseSseBufferResult {
  const items: FeedItem[] = [];
  let lastEventId: string | null = null;
  const parts = buffer.split('\n');
  const remainder = parts.pop() ?? '';

  let currentData: string | null = null;
  let currentId: string | null = null;

  const flushEvent = (): void => {
    if (currentData === null) {
      return;
    }
    const payload = currentData.trim();
    currentData = null;
    if (!payload || payload === 'keep-alive') {
      currentId = null;
      return;
    }
    try {
      items.push(JSON.parse(payload) as FeedItem);
      if (currentId) {
        lastEventId = currentId;
      }
    } catch {
      // Skip malformed events
    }
    currentId = null;
  };

  for (const line of parts) {
    if (line === '') {
      flushEvent();
      continue;
    }
    if (line.startsWith('id:')) {
      currentId = line.slice(3).trim();
      continue;
    }
    if (line.startsWith('data:')) {
      const value = line.slice(5).trim();
      currentData = currentData === null ? value : `${currentData}\n${value}`;
    }
  }

  return { items, remainder, lastEventId };
}

/**
 * Parses a Server-Sent Events text chunk into feed items (complete lines only).
 *
 * @param chunk Raw SSE text chunk from the stream.
 * @returns Parsed feed items from complete `data:` lines.
 */
export function parseSseChunk(chunk: string): FeedItem[] {
  // Ensure a terminating blank line so one-shot chunks ending with a single
  // `data:` line still flush into a complete event.
  const normalized = chunk.endsWith('\n\n')
    ? chunk
    : chunk.endsWith('\n')
      ? `${chunk}\n`
      : `${chunk}\n\n`;
  return parseSseBuffer(normalized).items;
}

/**
 * Options for SSE subscription with optional resume.
 */
export interface SubscribeFeedSseOptions {
  /** Full SSE endpoint URL. */
  url: string;
  /** Bearer access token. */
  token: string;
  /** Callback invoked for each parsed feed item. */
  onItem: (item: FeedItem) => void;
  /** AbortSignal to tear down the connection. */
  signal: AbortSignal;
  /** Last event id for replay (`Last-Event-ID`). */
  lastEventId?: string | null;
  /** Invoked when an SSE `id:` is observed. */
  onEventId?: (eventId: string) => void;
}

/**
 * Subscribes to an SSE endpoint and invokes the callback for each feed item.
 * Uses XMLHttpRequest for React Native streaming compatibility.
 *
 * @param options Subscription options.
 * @returns Promise that resolves when the stream ends or is aborted.
 */
export function subscribeFeedSse(options: SubscribeFeedSseOptions): Promise<void> {
  const { url, token, onItem, signal, lastEventId, onEventId } = options;

  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    let processedLength = 0;
    let lineBuffer = '';

    const cleanup = (): void => {
      xhr.abort();
    };

    signal.addEventListener('abort', cleanup);

    xhr.open('GET', url);
    xhr.setRequestHeader('Authorization', `Bearer ${token}`);
    xhr.setRequestHeader('Accept', 'text/event-stream');
    xhr.setRequestHeader('Cache-Control', 'no-cache');
    if (lastEventId) {
      xhr.setRequestHeader('Last-Event-ID', lastEventId);
    }

    xhr.onprogress = (): void => {
      const newText = xhr.responseText.slice(processedLength);
      processedLength = xhr.responseText.length;
      lineBuffer += newText;
      const parsed = parseSseBuffer(lineBuffer);
      lineBuffer = parsed.remainder;
      if (parsed.lastEventId && onEventId) {
        onEventId(parsed.lastEventId);
      }
      for (const item of parsed.items) {
        onItem(item);
      }
    };

    xhr.onload = (): void => {
      signal.removeEventListener('abort', cleanup);
      if (lineBuffer.length > 0) {
        const parsed = parseSseBuffer(`${lineBuffer}\n`);
        for (const item of parsed.items) {
          onItem(item);
        }
        if (parsed.lastEventId && onEventId) {
          onEventId(parsed.lastEventId);
        }
      }
      resolve();
    };

    xhr.onerror = (): void => {
      signal.removeEventListener('abort', cleanup);
      if (signal.aborted) {
        resolve();
        return;
      }
      reject(new Error('SSE connection failed'));
    };

    xhr.onabort = (): void => {
      signal.removeEventListener('abort', cleanup);
      resolve();
    };

    xhr.send();
  });
}
