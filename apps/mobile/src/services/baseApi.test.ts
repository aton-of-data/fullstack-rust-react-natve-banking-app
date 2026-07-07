import { configureStore } from '@reduxjs/toolkit';
import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';

import { authReducer, setCredentials } from '@/features/auth';
import { transferSubmissionReducer } from '@/features/transfer-submission';
import { baseApi, ficusApi } from './baseApi';

function createTestStore() {
  return configureStore({
    reducer: {
      auth: authReducer,
      transferSubmission: transferSubmissionReducer,
      [baseApi.reducerPath]: baseApi.reducer,
    },
    middleware: (getDefaultMiddleware) => getDefaultMiddleware().concat(baseApi.middleware),
  });
}

function mockFetchResponse(body: unknown, status = 200): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: new Headers({ 'content-type': 'application/json' }),
    json: async () => body,
    text: async () => JSON.stringify(body),
    clone: function clone() {
      return mockFetchResponse(body, status);
    },
  } as Response;
}

describe('createTransfer mutation', () => {
  const fetchMock = vi.fn();

  beforeEach(() => {
    vi.stubGlobal('fetch', fetchMock);
    fetchMock.mockResolvedValue(
      mockFetchResponse({
        transfer_id: 't-1',
        status: 'COMPLETED',
        currency: 'USD',
        sender_balance_minor: '9900',
        created_at: '2025-01-01T00:00:00Z',
      }),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('sends Idempotency-Key header', async () => {
    const store = createTestStore();
    store.dispatch(
      setCredentials({ accessToken: 'token-abc', userId: 'u1', username: 'alice' }),
    );

    const result = await store.dispatch(
      ficusApi.endpoints.createTransfer.initiate({
        body: {
          recipient_username: 'bob',
          amount_minor: '100',
          currency: 'USD',
        },
        idempotencyKey: '550e8400-e29b-41d4-a716-446655440000',
      }),
    );

    expect(result.data?.transfer_id).toBe('t-1');
    expect(fetchMock).toHaveBeenCalled();
    const request = fetchMock.mock.calls[0]?.[0] as Request | string;
    const url = typeof request === 'string' ? request : request.url;
    const init =
      typeof request === 'string'
        ? (fetchMock.mock.calls[0]?.[1] as RequestInit)
        : undefined;
    expect(url).toContain('/v1/transfers');
    const headers =
      init?.headers instanceof Headers
        ? Object.fromEntries(init.headers.entries())
        : (init?.headers as Record<string, string> | undefined) ??
          Object.fromEntries((request as Request).headers.entries());
    expect(headers['Idempotency-Key'] ?? headers['idempotency-key']).toBe(
      '550e8400-e29b-41d4-a716-446655440000',
    );
    expect(headers.Authorization ?? headers.authorization).toBe('Bearer token-abc');
  });
});
