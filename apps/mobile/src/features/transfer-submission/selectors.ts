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
