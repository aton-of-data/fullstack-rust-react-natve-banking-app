import { describe, expect, it } from 'vitest';

import { mapTransferError } from './transferErrors';

describe('mapTransferError', () => {
  it('maps insufficient funds to non-retryable guidance', () => {
    const mapped = mapTransferError({
      status: 422,
      data: { code: 'INSUFFICIENT_FUNDS', message: 'Insufficient funds' },
    });
    expect(mapped.code).toBe('INSUFFICIENT_FUNDS');
    expect(mapped.retryable).toBe(false);
  });

  it('maps idempotency conflict to non-retryable guidance', () => {
    const mapped = mapTransferError({
      status: 409,
      data: { code: 'IDEMPOTENCY_CONFLICT', message: 'Conflict' },
    });
    expect(mapped.code).toBe('IDEMPOTENCY_CONFLICT');
    expect(mapped.retryable).toBe(false);
  });

  it('maps network errors to retryable unknown outcome', () => {
    const mapped = mapTransferError({ status: 'FETCH_ERROR' });
    expect(mapped.retryable).toBe(true);
    expect(mapped.code).toBe('NETWORK_ERROR');
  });

  it('maps unauthorized to non-retryable session guidance', () => {
    const mapped = mapTransferError({ status: 401, data: { code: 'UNAUTHORIZED' } });
    expect(mapped.retryable).toBe(false);
  });

  it('maps recipient not found and validation errors', () => {
    expect(mapTransferError({ status: 404, data: { code: 'RECIPIENT_NOT_FOUND' } }).code).toBe(
      'RECIPIENT_NOT_FOUND',
    );
    expect(mapTransferError({ status: 422, data: { message: 'bad amount' } }).code).toBe(
      'VALIDATION_ERROR',
    );
  });

  it('maps timeout and server errors as retryable', () => {
    expect(mapTransferError({ status: 'TIMEOUT_ERROR' }).retryable).toBe(true);
    expect(mapTransferError({ status: 503, data: { code: 'SERVER_ERROR' } }).retryable).toBe(true);
  });

  it('maps unknown object shapes safely', () => {
    expect(mapTransferError({ status: 418 }).retryable).toBe(false);
    expect(mapTransferError('boom').code).toBe('UNKNOWN_ERROR');
  });
});
