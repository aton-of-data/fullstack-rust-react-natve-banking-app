import { describe, expect, it } from 'vitest';

import {
  beginTransferAttempt,
  generateIdempotencyKey,
  resetSubmission,
  submissionFailed,
  submissionStarted,
  submissionSucceeded,
  transferSubmissionReducer,
} from './transferSubmissionSlice';

const payload = { amountInput: '10.00', description: 'coffee' };

describe('transferSubmissionReducer', () => {
  it('generates idempotency key when transfer attempt begins', () => {
    const state = transferSubmissionReducer(undefined, beginTransferAttempt(payload));
    expect(state.idempotencyKey).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );
    expect(state.status).toBe('idle');
    expect(state.lockedAmountInput).toBe('10.00');
  });

  it('reuses the same key during retryable unknown outcome', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt(payload));
    const key = state.idempotencyKey;
    state = transferSubmissionReducer(state, submissionStarted());
    state = transferSubmissionReducer(
      state,
      submissionFailed({
        code: 'NETWORK_ERROR',
        message: 'Connection lost',
        retryable: true,
      }),
    );
    expect(state.status).toBe('unknown_outcome');
    expect(state.idempotencyKey).toBe(key);
    expect(state.retryable).toBe(true);
  });

  it('preserves key when beginTransferAttempt is called again while key is active', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt(payload));
    const key = state.idempotencyKey;
    state = transferSubmissionReducer(
      state,
      submissionFailed({
        code: 'NETWORK_ERROR',
        message: 'timeout',
        retryable: true,
      }),
    );
    state = transferSubmissionReducer(
      state,
      beginTransferAttempt({ amountInput: '99.00', description: 'changed' }),
    );
    expect(state.idempotencyKey).toBe(key);
    expect(state.lockedAmountInput).toBe('10.00');
  });

  it('rotates key after reset for a new transfer', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt(payload));
    const firstKey = state.idempotencyKey;
    state = transferSubmissionReducer(state, submissionSucceeded('transfer-1'));
    state = transferSubmissionReducer(state, resetSubmission());
    state = transferSubmissionReducer(state, beginTransferAttempt(payload));
    expect(state.idempotencyKey).not.toBe(firstKey);
  });

  it('marks idempotency conflict as non-retryable', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt(payload));
    state = transferSubmissionReducer(
      state,
      submissionFailed({
        code: 'IDEMPOTENCY_CONFLICT',
        message: 'Conflict',
        retryable: false,
      }),
    );
    expect(state.status).toBe('failed');
    expect(state.retryable).toBe(false);
  });

  it('generateIdempotencyKey is stable format in tests', () => {
    const key = generateIdempotencyKey();
    expect(key.length).toBeGreaterThanOrEqual(36);
  });
});
