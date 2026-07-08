import {
  createApi,
  fetchBaseQuery,
  type BaseQueryFn,
  type FetchArgs,
  type FetchBaseQueryError,
} from '@reduxjs/toolkit/query/react';
import type {
  BalanceResponse,
  FeedItem,
  FeedPageResponse,
  LedgerPageResponse,
  LoginRequest,
  LoginResponse,
  MeResponse,
  TransferRequest,
  TransferResponse,
  UserSearchPageResponse,
} from '@ficus/contracts';

import { clearCredentials } from '@/features/auth/authSlice';
import { resetForm } from '@/features/transfer-form/transferFormSlice';
import { resetSubmission } from '@/features/transfer-submission/transferSubmissionSlice';
import type { RootState } from '@/store';
import { getApiBaseUrl } from './config';
import { subscribeFeedSse } from './sse';

/**
 * Transfer mutation argument including Redux-owned idempotency key.
 */
export interface CreateTransferArg {
  /** Transfer request body. */
  body: TransferRequest;
  /** Idempotency key header value. */
  idempotencyKey: string;
}

/**
 * RTK Query tag types for cache invalidation.
 */
export const apiTags = ['Auth', 'Balance', 'Feed', 'Users', 'Transfers'] as const;

const rawBaseQuery = fetchBaseQuery({
  baseUrl: getApiBaseUrl(),
  prepareHeaders: (headers, { getState }) => {
    const token = (getState() as RootState).auth.accessToken;
    if (token) {
      headers.set('Authorization', `Bearer ${token}`);
    }
    headers.set('Content-Type', 'application/json');
    return headers;
  },
});

/**
 * Base query that forces logout on 401 for authenticated endpoints.
 *
 * @param args Request args.
 * @param api RTK Query API helpers.
 * @param extra Extra options.
 * @returns Query result.
 */
const baseQueryWithAuth: BaseQueryFn<string | FetchArgs, unknown, FetchBaseQueryError> = async (
  args,
  api,
  extra,
) => {
  const result = await rawBaseQuery(args, api, extra);
  const url = typeof args === 'string' ? args : args.url;
  const isLogin = url.includes('/v1/auth/login');
  if (result.error && result.error.status === 401 && !isLogin) {
    api.dispatch(clearCredentials());
    api.dispatch(resetSubmission());
    api.dispatch(resetForm());
    api.dispatch(baseApi.util.resetApiState());
  }
  return result;
};

/**
 * Base RTK Query API with auth header injection and 401 logout.
 */
export const baseApi = createApi({
  reducerPath: 'api',
  baseQuery: baseQueryWithAuth,
  tagTypes: [...apiTags],
  endpoints: () => ({}),
});

const SSE_MAX_BACKOFF_MS = 30_000;

/**
 * Sleeps until timeout or abort.
 *
 * @param ms Delay in milliseconds.
 * @param signal Abort signal.
 * @returns Promise that resolves after delay or abort.
 */
function delay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.aborted) {
      resolve();
      return;
    }
    const timer = setTimeout(() => {
      signal.removeEventListener('abort', onAbort);
      resolve();
    }, ms);
    const onAbort = (): void => {
      clearTimeout(timer);
      resolve();
    };
    signal.addEventListener('abort', onAbort, { once: true });
  });
}

/**
 * Injected API endpoints for auth, users, accounts, transfers, and feed.
 */
export const ficusApi = baseApi.injectEndpoints({
  endpoints: (builder) => ({
    /**
     * Authenticates a user and returns a JWT access token.
     */
    login: builder.mutation<LoginResponse, LoginRequest>({
      query: (body) => ({
        url: '/v1/auth/login',
        method: 'POST',
        body,
      }),
      invalidatesTags: ['Auth', 'Balance', 'Feed'],
    }),

    /**
     * Logs out the current session on the server.
     */
    logout: builder.mutation<void, void>({
      query: () => ({
        url: '/v1/auth/logout',
        method: 'POST',
      }),
      invalidatesTags: ['Auth', 'Balance', 'Feed'],
    }),

    /**
     * Returns the authenticated user's profile.
     */
    getMe: builder.query<MeResponse, void>({
      query: () => '/v1/auth/me',
      providesTags: ['Auth'],
    }),

    /**
     * Searches users by username prefix.
     */
    searchUsers: builder.query<UserSearchPageResponse, { query: string; cursor?: string }>({
      query: ({ query, cursor }) => ({
        url: '/v1/users',
        params: { query, ...(cursor ? { cursor } : {}) },
      }),
      providesTags: ['Users'],
    }),

    /**
     * Returns the authenticated user's account balance.
     */
    getBalance: builder.query<BalanceResponse, void>({
      query: () => '/v1/accounts/me/balance',
      providesTags: ['Balance'],
    }),

    /**
     * Returns paginated ledger entries for the authenticated user.
     */
    getLedger: builder.query<LedgerPageResponse, { cursor?: string } | void>({
      query: (arg) =>
        arg?.cursor
          ? { url: '/v1/accounts/me/ledger', params: { cursor: arg.cursor } }
          : '/v1/accounts/me/ledger',
      providesTags: ['Balance'],
    }),

    /**
     * Creates a money transfer to another user.
     */
    createTransfer: builder.mutation<TransferResponse, CreateTransferArg>({
      query: ({ body, idempotencyKey }) => ({
        url: '/v1/transfers',
        method: 'POST',
        body,
        headers: {
          'Idempotency-Key': idempotencyKey,
        },
      }),
      invalidatesTags: ['Balance', 'Feed', 'Transfers'],
    }),

    /**
     * Returns the global transaction feed with live SSE updates.
     */
    getFeed: builder.query<FeedItem[], void>({
      query: () => '/v1/feed',
      transformResponse: (response: FeedPageResponse) => response.items,
      providesTags: ['Feed'],
      async onCacheEntryAdded(_arg, api) {
        await api.cacheDataLoaded;
        const token = (api.getState() as RootState).auth.accessToken;
        if (!token) {
          return;
        }

        const controller = new AbortController();
        const url = `${getApiBaseUrl()}/v1/feed/stream`;
        let lastEventId: string | null = null;
        let attempt = 0;

        const run = async (): Promise<void> => {
          while (!controller.signal.aborted) {
            try {
              await subscribeFeedSse({
                url,
                token,
                signal: controller.signal,
                lastEventId,
                onEventId: (eventId) => {
                  lastEventId = eventId;
                },
                onItem: (item) => {
                  api.updateCachedData((draft) => {
                    const exists = draft.some(
                      (existing) => existing.transfer_id === item.transfer_id,
                    );
                    if (!exists) {
                      draft.unshift(item);
                    }
                  });
                },
              });
              // Clean stream end (or abort): reconnect with backoff unless torn down.
              if (controller.signal.aborted) {
                return;
              }
              attempt = 0;
              await delay(1000, controller.signal);
            } catch {
              if (controller.signal.aborted) {
                return;
              }
              attempt += 1;
              const backoff = Math.min(1000 * 2 ** (attempt - 1), SSE_MAX_BACKOFF_MS);
              await delay(backoff, controller.signal);
            }
          }
        };

        void run();
        await api.cacheEntryRemoved;
        controller.abort();
      },
    }),
  }),
});

export const {
  useLoginMutation,
  useLogoutMutation,
  useGetMeQuery,
  useSearchUsersQuery,
  useGetBalanceQuery,
  useGetLedgerQuery,
  useCreateTransferMutation,
  useGetFeedQuery,
} = ficusApi;
