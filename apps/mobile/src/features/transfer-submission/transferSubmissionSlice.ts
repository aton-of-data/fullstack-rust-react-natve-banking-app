import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

/**
 * Transfer submission lifecycle status.
 */
export type TransferSubmissionStatus =
  'idle' | 'submitting' | 'unknown_outcome' | 'succeeded' | 'failed';

/**
 * Redux-owned transfer submission state for retry-safe idempotency.
 */
export interface TransferSubmissionState {
  /** Active idempotency key for the current logical transfer attempt. */
  idempotencyKey: string | null;
  /** Submission lifecycle status. */
  status: TransferSubmissionStatus;
  /** Last successful transfer identifier. */
  lastTransferId: string | null;
  /** Whether the client may retry with the same key and payload. */
  retryable: boolean;
  /** Machine-readable API error code when present. */
  errorCode: string | null;
  /** User-facing error message. */
  errorMessage: string | null;
}

const initialState: TransferSubmissionState = {
  idempotencyKey: null,
  status: 'idle',
  lastTransferId: null,
  retryable: false,
  errorCode: null,
  errorMessage: null,
};

/**
 * Generates a UUID-compatible idempotency key.
 *
 * @returns New idempotency key string.
 */
export function generateIdempotencyKey(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `00000000-0000-4000-8000-${Date.now().toString(16).padStart(12, '0')}`;
}

/**
 * Redux slice managing transfer submission idempotency lifecycle.
 */
export const transferSubmissionSlice = createSlice({
  name: 'transferSubmission',
  initialState,
  reducers: {
    /**
     * Begins a new transfer attempt by generating a fresh idempotency key.
     *
     * @param state Current submission state.
     * @returns Updated submission state.
     */
    beginTransferAttempt(state) {
      state.idempotencyKey = generateIdempotencyKey();
      state.status = 'idle';
      state.lastTransferId = null;
      state.retryable = false;
      state.errorCode = null;
      state.errorMessage = null;
    },

    /**
     * Marks the transfer as actively submitting.
     *
     * @param state Current submission state.
     */
    submissionStarted(state) {
      state.status = 'submitting';
      state.errorCode = null;
      state.errorMessage = null;
    },

    /**
     * Records a successful transfer outcome.
     *
     * @param state Current submission state.
     * @param action Transfer identifier from API response.
     */
    submissionSucceeded(state, action: PayloadAction<string>) {
      state.status = 'succeeded';
      state.lastTransferId = action.payload;
      state.retryable = false;
      state.errorCode = null;
      state.errorMessage = null;
    },

    /**
     * Records a failed transfer with mapped error metadata.
     *
     * @param state Current submission state.
     * @param action Error code, message, and retry eligibility.
     */
    submissionFailed(
      state,
      action: PayloadAction<{ code: string | null; message: string; retryable: boolean }>,
    ) {
      state.status = action.payload.retryable ? 'unknown_outcome' : 'failed';
      state.retryable = action.payload.retryable;
      state.errorCode = action.payload.code;
      state.errorMessage = action.payload.message;
    },

    /**
     * Clears submission state after completion or explicit reset.
     *
     * @returns Initial submission state.
     */
    resetSubmission() {
      return { ...initialState };
    },
  },
});

/** Transfer submission action creators. */
export const {
  beginTransferAttempt,
  submissionStarted,
  submissionSucceeded,
  submissionFailed,
  resetSubmission,
} = transferSubmissionSlice.actions;

/** Transfer submission reducer. */
export const transferSubmissionReducer = transferSubmissionSlice.reducer;
