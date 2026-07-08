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
  /** Snapshot of amount input locked with the idempotency key. */
  lockedAmountInput: string | null;
  /** Snapshot of description locked with the idempotency key. */
  lockedDescription: string | null;
}

const initialState: TransferSubmissionState = {
  idempotencyKey: null,
  status: 'idle',
  lastTransferId: null,
  retryable: false,
  errorCode: null,
  errorMessage: null,
  lockedAmountInput: null,
  lockedDescription: null,
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
 * Payload for beginning a transfer attempt with form snapshot.
 */
export interface BeginTransferAttemptPayload {
  /** Amount input (major-unit decimal string) locked to this key. */
  amountInput: string;
  /** Optional description locked to this key. */
  description: string;
}

/**
 * Redux slice managing transfer submission idempotency lifecycle.
 */
export const transferSubmissionSlice = createSlice({
  name: 'transferSubmission',
  initialState,
  reducers: {
    /**
     * Begins a new transfer attempt only when no key is active.
     * Preserves an existing key (and locked payload) so Back→Review cannot rotate keys after unknown outcomes.
     *
     * @param state Current submission state.
     * @param action Amount and description snapshot for the attempt.
     */
    beginTransferAttempt(state, action: PayloadAction<BeginTransferAttemptPayload>) {
      if (state.idempotencyKey) {
        return;
      }
      state.idempotencyKey = generateIdempotencyKey();
      state.status = 'idle';
      state.lastTransferId = null;
      state.retryable = false;
      state.errorCode = null;
      state.errorMessage = null;
      state.lockedAmountInput = action.payload.amountInput;
      state.lockedDescription = action.payload.description;
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
     * Cleared acknowledgement after success UI is shown — keeps lastTransferId until reset.
     *
     * @param state Current submission state.
     */
    acknowledgeSuccess(state) {
      state.status = 'succeeded';
    },

    /**
     * Clears submission state after completion or explicit cancel / new transfer.
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
  acknowledgeSuccess,
  resetSubmission,
} = transferSubmissionSlice.actions;

/** Transfer submission reducer. */
export const transferSubmissionReducer = transferSubmissionSlice.reducer;
