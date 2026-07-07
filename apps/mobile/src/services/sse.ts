import type { FeedItem } from "@ficus/contracts";

/**
 * Parses a Server-Sent Events text chunk into feed items.
 *
 * @param chunk Raw SSE text chunk from the stream.
 * @returns Parsed feed items from complete `data:` lines.
 */
export function parseSseChunk(chunk: string): FeedItem[] {
  const items: FeedItem[] = [];
  const lines = chunk.split("\n");

  for (const line of lines) {
    if (!line.startsWith("data:")) {
      continue;
    }
    const payload = line.slice(5).trim();
    if (!payload || payload === "keep-alive") {
      continue;
    }
    try {
      items.push(JSON.parse(payload) as FeedItem);
    } catch {
      // Skip malformed events
    }
  }

  return items;
}

/**
 * Subscribes to an SSE endpoint and invokes the callback for each feed item.
 * Uses XMLHttpRequest for React Native streaming compatibility.
 *
 * @param url Full SSE endpoint URL.
 * @param token Bearer access token.
 * @param onItem Callback invoked for each parsed feed item.
 * @param signal AbortSignal to tear down the connection.
 */
export function subscribeFeedSse(
  url: string,
  token: string,
  onItem: (item: FeedItem) => void,
  signal: AbortSignal,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    let buffer = "";

    const cleanup = (): void => {
      xhr.abort();
    };

    signal.addEventListener("abort", cleanup);

    xhr.open("GET", url);
    xhr.setRequestHeader("Authorization", `Bearer ${token}`);
    xhr.setRequestHeader("Accept", "text/event-stream");
    xhr.setRequestHeader("Cache-Control", "no-cache");

    xhr.onprogress = (): void => {
      const newText = xhr.responseText.slice(buffer.length);
      buffer = xhr.responseText;
      const items = parseSseChunk(newText);
      for (const item of items) {
        onItem(item);
      }
    };

    xhr.onload = (): void => {
      signal.removeEventListener("abort", cleanup);
      resolve();
    };

    xhr.onerror = (): void => {
      signal.removeEventListener("abort", cleanup);
      if (signal.aborted) {
        resolve();
        return;
      }
      reject(new Error("SSE connection failed"));
    };

    xhr.onabort = (): void => {
      signal.removeEventListener("abort", cleanup);
      resolve();
    };

    xhr.send();
  });
}
