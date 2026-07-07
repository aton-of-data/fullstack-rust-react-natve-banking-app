/**
 * Maps transfer API errors to user-facing messages and retry policy.
 */
export interface MappedTransferError {
  /** Machine-readable API error code when available. */
  code: string | null;
  /** Safe user-facing message. */
  message: string;
  /** Whether the same idempotency key may be reused. */
  retryable: boolean;
}

/**
 * RTK Query error shape with optional status and data payload.
 */
interface RtkQueryError {
  status?: number | string;
  data?: { code?: string; message?: string };
}

/**
 * Maps a transfer mutation error to UI state.
 *
 * @param error Error thrown by RTK Query unwrap.
 * @returns Mapped error with retry guidance.
 */
export function mapTransferError(error: unknown): MappedTransferError {
  if (error && typeof error === 'object' && 'status' in error) {
    const rtkError = error as RtkQueryError;
    const code = rtkError.data?.code ?? null;
    const apiMessage = rtkError.data?.message;

    if (rtkError.status === 401) {
      return {
        code: code ?? 'UNAUTHORIZED',
        message: 'Your session expired. Please sign in again.',
        retryable: false,
      };
    }
    if (rtkError.status === 409 || code === 'IDEMPOTENCY_CONFLICT') {
      return {
        code: 'IDEMPOTENCY_CONFLICT',
        message: 'This transfer conflicts with a previous request. Contact support if needed.',
        retryable: false,
      };
    }
    if (rtkError.status === 422 && code === 'INSUFFICIENT_FUNDS') {
      return {
        code: 'INSUFFICIENT_FUNDS',
        message: 'Insufficient funds for this transfer. Adjust the amount or add funds.',
        retryable: false,
      };
    }
    if (rtkError.status === 404 || code === 'RECIPIENT_NOT_FOUND') {
      return {
        code: 'RECIPIENT_NOT_FOUND',
        message: 'Recipient not found. Choose a different recipient.',
        retryable: false,
      };
    }
    if (rtkError.status === 422) {
      return {
        code: code ?? 'VALIDATION_ERROR',
        message: apiMessage ?? 'Check the transfer details and try again.',
        retryable: false,
      };
    }
    if (rtkError.status === 'FETCH_ERROR' || rtkError.status === 'TIMEOUT_ERROR') {
      return {
        code: 'NETWORK_ERROR',
        message: 'Connection lost. Retry to complete the transfer safely.',
        retryable: true,
      };
    }
    if (typeof rtkError.status === 'number' && rtkError.status >= 500) {
      return {
        code: code ?? 'SERVER_ERROR',
        message: 'Server error. You can retry without changing the amount.',
        retryable: true,
      };
    }

    return {
      code,
      message: apiMessage ?? 'Transfer failed. Please try again.',
      retryable: false,
    };
  }

  return {
    code: 'UNKNOWN_ERROR',
    message: 'Transfer failed. Please try again.',
    retryable: true,
  };
}
