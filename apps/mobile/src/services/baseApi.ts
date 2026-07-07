import { createApi, fetchBaseQuery } from "@reduxjs/toolkit/query/react";
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
} from "@ficus/contracts";

import type { RootState } from "@/store";
import { getApiBaseUrl } from "./config";
import { subscribeFeedSse } from "./sse";

/**
 * RTK Query tag types for cache invalidation.
 */
export const apiTags = [
  "Auth",
  "Balance",
  "Feed",
  "Users",
  "Transfers",
] as const;

/**
 * Base RTK Query API with auth header injection.
 */
export const baseApi = createApi({
  reducerPath: "api",
  baseQuery: fetchBaseQuery({
    baseUrl: getApiBaseUrl(),
    prepareHeaders: (headers, { getState }) => {
      const token = (getState() as RootState).auth.accessToken;
      if (token) {
        headers.set("Authorization", `Bearer ${token}`);
      }
      headers.set("Content-Type", "application/json");
      return headers;
    },
  }),
  tagTypes: [...apiTags],
  endpoints: () => ({}),
});

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
        url: "/v1/auth/login",
        method: "POST",
        body,
      }),
      invalidatesTags: ["Auth", "Balance", "Feed"],
    }),

    /**
     * Logs out the current session on the server.
     */
    logout: builder.mutation<void, void>({
      query: () => ({
        url: "/v1/auth/logout",
        method: "POST",
      }),
      invalidatesTags: ["Auth", "Balance", "Feed"],
    }),

    /**
     * Returns the authenticated user's profile.
     */
    getMe: builder.query<MeResponse, void>({
      query: () => "/v1/auth/me",
      providesTags: ["Auth"],
    }),

    /**
     * Searches users by username prefix.
     */
    searchUsers: builder.query<
      UserSearchPageResponse,
      { query: string; cursor?: string }
    >({
      query: ({ query, cursor }) => ({
        url: "/v1/users",
        params: { query, ...(cursor ? { cursor } : {}) },
      }),
      providesTags: ["Users"],
    }),

    /**
     * Returns the authenticated user's account balance.
     */
    getBalance: builder.query<BalanceResponse, void>({
      query: () => "/v1/accounts/me/balance",
      providesTags: ["Balance"],
    }),

    /**
     * Returns paginated ledger entries for the authenticated user.
     */
    getLedger: builder.query<LedgerPageResponse, { cursor?: string } | void>({
      query: (arg) =>
        arg?.cursor
          ? { url: "/v1/accounts/me/ledger", params: { cursor: arg.cursor } }
          : "/v1/accounts/me/ledger",
      providesTags: ["Balance"],
    }),

    /**
     * Creates a money transfer to another user.
     */
    createTransfer: builder.mutation<TransferResponse, TransferRequest>({
      query: (body) => ({
        url: "/v1/transfers",
        method: "POST",
        body,
      }),
      invalidatesTags: ["Balance", "Feed", "Transfers"],
    }),

    /**
     * Returns the global transaction feed with live SSE updates.
     */
    getFeed: builder.query<FeedItem[], void>({
      query: () => "/v1/feed",
      transformResponse: (response: FeedPageResponse) => response.items,
      providesTags: ["Feed"],
      async onCacheEntryAdded(_arg, api) {
        await api.cacheDataLoaded;
        const token = (api.getState() as RootState).auth.accessToken;
        if (!token) {
          return;
        }

        const controller = new AbortController();
        const url = `${getApiBaseUrl()}/v1/feed/stream`;

        void subscribeFeedSse(
          url,
          token,
          (item) => {
            api.updateCachedData((draft) => {
              const exists = draft.some(
                (existing) => existing.transfer_id === item.transfer_id,
              );
              if (!exists) {
                draft.unshift(item);
              }
            });
          },
          controller.signal,
        ).catch(() => {
          // SSE errors are non-fatal; REST cache remains valid
        });

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
