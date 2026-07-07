/**
 * ISO 8601 UTC timestamp string from the API.
 */
export type IsoDateTime = string;

/**
 * UUID string from the API.
 */
export type Uuid = string;

/**
 * Login request body (`POST /auth/login`).
 */
export interface LoginRequest {
  /** Username credential. */
  username: string;
  /** Plain-text password credential. */
  password: string;
}

/**
 * Successful login response.
 */
export interface LoginResponse {
  /** Bearer access token. */
  access_token: string;
  /** Authenticated user identifier. */
  user_id: Uuid;
  /** Authenticated username. */
  username: string;
}

/**
 * Current user profile (`GET /auth/me`).
 */
export interface MeResponse {
  /** Authenticated user identifier. */
  user_id: Uuid;
  /** Authenticated username. */
  username: string;
}

/**
 * User search result item.
 */
export interface UserSearchItem {
  /** Matched user identifier. */
  user_id: Uuid;
  /** Matched username. */
  username: string;
}

/**
 * Account balance response.
 */
export interface BalanceResponse {
  /** Balance in integer minor units (wire string). */
  balance_minor: string;
  /** ISO 4217 currency code. */
  currency: string;
}

/**
 * Ledger direction values returned by the API.
 */
export type LedgerDirection = 'DEBIT' | 'CREDIT' | string;

/**
 * Single ledger entry in account history.
 */
export interface LedgerItemResponse {
  /** Ledger entry identifier. */
  entry_id: Uuid;
  /** Related transfer identifier. */
  transfer_id: Uuid;
  /** Entry amount in integer minor units (wire string). */
  amount_minor: string;
  /** Debit or credit direction. */
  direction: LedgerDirection;
  /** ISO 4217 currency code. */
  currency: string;
  /** Entry creation timestamp (UTC). */
  created_at: IsoDateTime;
}

/**
 * Transfer creation request body (`POST /transfers`).
 */
export interface TransferRequest {
  /** Recipient username. */
  recipient_username: string;
  /** Transfer amount in integer minor units (wire string). */
  amount_minor: string;
  /** ISO 4217 currency code. */
  currency: string;
  /** Optional transfer memo. */
  description?: string;
}

/**
 * Transfer status values returned by the API.
 */
export type TransferStatus = 'COMPLETED' | 'DECLINED' | string;

/**
 * Transfer creation response.
 */
export interface TransferResponse {
  /** Transfer identifier. */
  transfer_id: Uuid;
  /** Final transfer status. */
  status: TransferStatus;
  /** Sender balance after transfer in minor units (wire string). */
  sender_balance_minor: string;
  /** ISO 4217 currency code. */
  currency: string;
  /** Transfer creation timestamp (UTC). */
  created_at: IsoDateTime;
}

/**
 * Public feed item (REST and real-time payloads).
 */
export interface FeedItem {
  /** Transfer identifier. */
  transfer_id: Uuid;
  /** Sender username. */
  sender_username: string;
  /** Recipient username. */
  recipient_username: string;
  /** Transfer amount in integer minor units (wire string). */
  amount_minor: string;
  /** ISO 4217 currency code. */
  currency: string;
  /** Optional transfer memo. */
  description?: string | null;
  /** Transfer creation timestamp (UTC). */
  created_at: IsoDateTime;
}

/**
 * Cursor-paginated API response wrapper.
 */
export interface PageResponse<T> {
  /** Page items. */
  items: T[];
  /** Opaque cursor for the next page, if any. */
  next_cursor?: string | null;
}

/**
 * Paginated user search response.
 */
export type UserSearchPageResponse = PageResponse<UserSearchItem>;

/**
 * Paginated ledger response.
 */
export type LedgerPageResponse = PageResponse<LedgerItemResponse>;

/**
 * Paginated feed response.
 */
export type FeedPageResponse = PageResponse<FeedItem>;
