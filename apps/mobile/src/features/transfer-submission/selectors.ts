import type { TransferSubmissionState } from './transferSubmissionSlice';

/**
 * Selects the active idempotency key.
 *
 * @param state Transfer submission slice state.
 * @returns Idempotency key or null.
 */
export function selectIdempotencyKey(state: TransferSubmissionState): string | null {
  return state.idempotencyKey;
}

/**
 * Selects submission lifecycle status.
 *
 * @param state Transfer submission slice state.
 * @returns Submission status.
 */
export function selectSubmissionStatus(
  state: TransferSubmissionState,
): TransferSubmissionState['status'] {
  return state.status;
}

/**
 * Selects whether retry is permitted with the current key.
 *
 * @param state Transfer submission slice state.
 * @returns Retry eligibility flag.
 */
export function selectSubmissionRetryable(state: TransferSubmissionState): boolean {
  return state.retryable;
}

/**
 * Selects the mapped submission error message.
 *
 * @param state Transfer submission slice state.
 * @returns Error message or null.
 */
export function selectSubmissionErrorMessage(state: TransferSubmissionState): string | null {
  return state.errorMessage;
}

/**
 * Selects the mapped API error code.
 *
 * @param state Transfer submission slice state.
 * @returns Error code or null.
 */
export function selectSubmissionErrorCode(state: TransferSubmissionState): string | null {
  return state.errorCode;
}

/**
 * Selects the last successful transfer id.
 *
 * @param state Transfer submission slice state.
 * @returns Transfer id or null.
 */
export function selectLastTransferId(state: TransferSubmissionState): string | null {
  return state.lastTransferId;
}

/**
 * Selects whether an active attempt locks form edits.
 *
 * @param state Transfer submission slice state.
 * @returns True when an idempotency key is held.
 */
export function selectHasActiveAttempt(state: TransferSubmissionState): boolean {
  return state.idempotencyKey !== null;
}

/**
 * Selects locked amount input for the active attempt.
 *
 * @param state Transfer submission slice state.
 * @returns Locked amount or null.
 */
export function selectLockedAmountInput(state: TransferSubmissionState): string | null {
  return state.lockedAmountInput;
}

/**
 * Selects locked description for the active attempt.
 *
 * @param state Transfer submission slice state.
 * @returns Locked description or null.
 */
export function selectLockedDescription(state: TransferSubmissionState): string | null {
  return state.lockedDescription;
}
