import { describe, expect, it } from 'vitest';

import {
  selectIdempotencyKey,
  selectSubmissionErrorCode,
  selectSubmissionErrorMessage,
  selectSubmissionRetryable,
  selectSubmissionStatus,
} from './selectors';
import {
  beginTransferAttempt,
  submissionFailed,
  submissionStarted,
  submissionSucceeded,
  transferSubmissionReducer,
} from './transferSubmissionSlice';

describe('transfer submission selectors', () => {
  it('exposes idempotency and lifecycle fields', () => {
    let state = transferSubmissionReducer(undefined, beginTransferAttempt());
    expect(selectIdempotencyKey(state)).toBeTruthy();
    const key = selectIdempotencyKey(state);

    state = transferSubmissionReducer(state, submissionStarted());
    expect(selectSubmissionStatus(state)).toBe('submitting');

    state = transferSubmissionReducer(
      state,
      submissionFailed({
        code: 'NETWORK_ERROR',
        message: 'timeout',
        retryable: true,
      }),
    );
    expect(selectSubmissionStatus(state)).toBe('unknown_outcome');
    expect(selectSubmissionRetryable(state)).toBe(true);
    expect(selectSubmissionErrorMessage(state)).toBe('timeout');
    expect(selectSubmissionErrorCode(state)).toBe('NETWORK_ERROR');
    expect(selectIdempotencyKey(state)).toBe(key);

    state = transferSubmissionReducer(state, submissionSucceeded('transfer-1'));
    expect(selectSubmissionStatus(state)).toBe('succeeded');
  });
});
