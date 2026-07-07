import { describe, expect, it } from 'vitest';

import {
  beginTransferAttempt,
  resetSubmission,
  submissionFailed,
  submissionStarted,
  submissionSucceeded,
  transferSubmissionReducer,
} from '@/features/transfer-submission';
import {
  goToConfirm,
  selectRecipient,
  setAmountInput,
  transferFormReducer,
} from '@/features/transfer-form';

describe('transfer confirmation workflow state', () => {
  it('generates key on confirm step and preserves it on retry', () => {
    let form = transferFormReducer(
      undefined,
      selectRecipient({ userId: 'u1', username: 'bob' }),
    );
    form = transferFormReducer(form, setAmountInput('10.00'));
    form = transferFormReducer(form, goToConfirm());
    expect(form.step).toBe('confirm');

    let submission = transferSubmissionReducer(undefined, beginTransferAttempt());
    const key = submission.idempotencyKey;
    submission = transferSubmissionReducer(submission, submissionStarted());
    submission = transferSubmissionReducer(
      submission,
      submissionFailed({
        code: 'NETWORK_ERROR',
        message: 'timeout',
        retryable: true,
      }),
    );
    expect(submission.idempotencyKey).toBe(key);
    expect(submission.status).toBe('unknown_outcome');
  });

  it('issues new key after successful completion and reset', () => {
    let submission = transferSubmissionReducer(undefined, beginTransferAttempt());
    const first = submission.idempotencyKey;
    submission = transferSubmissionReducer(submission, submissionSucceeded('transfer-99'));
    submission = transferSubmissionReducer(submission, resetSubmission());
    submission = transferSubmissionReducer(submission, beginTransferAttempt());
    expect(submission.idempotencyKey).not.toBe(first);
  });
});
