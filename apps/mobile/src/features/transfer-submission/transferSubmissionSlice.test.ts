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

describe('transferSubmissionReducer', () => {
  it('generates idempotency key when transfer attempt begins', () => {
    const state = transferSubmissionReducer(undefined, beginTransferAttempt());
    expect(state.idempotencyKey).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );
    expect(state.status).toBe('idle');
  });

  it('reuses the same key during retryable unknown outcome', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt());
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

  it('rotates key after reset for a new transfer', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt());
    const firstKey = state.idempotencyKey;
    state = transferSubmissionReducer(state, submissionSucceeded('transfer-1'));
    state = transferSubmissionReducer(state, resetSubmission());
    state = transferSubmissionReducer(state, beginTransferAttempt());
    expect(state.idempotencyKey).not.toBe(firstKey);
  });

  it('marks idempotency conflict as non-retryable', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt());
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
