import { describe, expect, it, vi, beforeEach } from 'vitest';

import { TransferConfirmation } from './TransferConfirmation';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

const createTransferMock = vi.fn();

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useCreateTransferMutation: () => [createTransferMock, { isLoading: false }],
  };
});

const baseForm = {
  step: 'confirm' as const,
  searchQuery: '',
  recipientUserId: 'u2',
  recipientUsername: 'bob',
  amountInput: '10.00',
  description: 'coffee',
  currency: 'USD',
  submitting: false,
  submitError: null,
};

describe('TransferConfirmation', () => {
  beforeEach(() => {
    createTransferMock.mockReset();
  });

  it('submits transfer with the Redux-owned idempotency key', async () => {
    createTransferMock.mockReturnValue({
      unwrap: () =>
        Promise.resolve({
          transfer_id: 'transfer-42',
          status: 'COMPLETED',
          currency: 'USD',
          sender_balance_minor: '9000',
          created_at: '2025-01-01T00:00:00Z',
        }),
    });
    const idempotencyKey = '550e8400-e29b-41d4-a716-446655440000';
    const { root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        auth: {
          status: 'authenticated',
          accessToken: 'tok',
          userId: 'u1',
          username: 'alice',
          hydrated: true,
        },
        transferForm: baseForm,
        transferSubmission: {
          idempotencyKey,
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
          lockedAmountInput: '10.00',
          lockedDescription: 'coffee',
        },
      },
    });

    expect(findByText(root, '$10.00')).toBeTruthy();
    findByLabel(root, 'Send money').props.onPress();

    await vi.waitFor(() => {
      expect(createTransferMock).toHaveBeenCalledWith({
        body: {
          recipient_username: 'bob',
          amount_minor: '1000',
          currency: 'USD',
          description: 'coffee',
        },
        idempotencyKey,
      });
    });
  });

  it('shows success confirmation with transfer reference', async () => {
    createTransferMock.mockReturnValue({
      unwrap: () =>
        Promise.resolve({
          transfer_id: 'transfer-42',
          status: 'COMPLETED',
          currency: 'USD',
          sender_balance_minor: '9000',
          created_at: '2025-01-01T00:00:00Z',
        }),
    });
    const { root, store } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: baseForm,
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440000',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
          lockedAmountInput: '10.00',
          lockedDescription: 'coffee',
        },
      },
    });

    findByLabel(root, 'Send money').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().transferSubmission.status).toBe('succeeded');
      expect(findByText(root, 'Transfer sent')).toBeTruthy();
      expect(findByLabel(root, 'Transfer reference transfer-42')).toBeTruthy();
    });
  });

  it('retries with the same idempotency key after unknown outcome', async () => {
    const idempotencyKey = '550e8400-e29b-41d4-a716-446655440099';
    createTransferMock.mockReturnValue({
      unwrap: () =>
        Promise.resolve({
          transfer_id: 'transfer-99',
          status: 'COMPLETED',
          currency: 'USD',
          sender_balance_minor: '9000',
          created_at: '2025-01-01T00:00:00Z',
        }),
    });

    const { root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: { ...baseForm, amountInput: '5.00', description: '' },
        transferSubmission: {
          idempotencyKey,
          status: 'unknown_outcome',
          lastTransferId: null,
          retryable: true,
          errorCode: 'NETWORK_ERROR',
          errorMessage: 'Connection lost. Retry to complete the transfer safely.',
          lockedAmountInput: '5.00',
          lockedDescription: '',
        },
      },
    });

    findByLabel(root, 'Retry transfer').props.onPress();
    await vi.waitFor(() => {
      expect(createTransferMock).toHaveBeenCalledWith({
        body: {
          recipient_username: 'bob',
          amount_minor: '500',
          currency: 'USD',
        },
        idempotencyKey,
      });
    });
  });

  it('shows retry guidance for unknown outcomes', () => {
    const { root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: { ...baseForm, amountInput: '5.00', description: '' },
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440001',
          status: 'unknown_outcome',
          lastTransferId: null,
          retryable: true,
          errorCode: 'NETWORK_ERROR',
          errorMessage: 'Request timed out',
          lockedAmountInput: '5.00',
          lockedDescription: '',
        },
      },
    });

    expect(findByText(root, 'Request timed out')).toBeTruthy();
    expect(findByLabel(root, 'Retry transfer')).toBeTruthy();
  });

  it('maps insufficient funds as non-retryable failure', async () => {
    createTransferMock.mockImplementation(() => ({
      unwrap: () =>
        Promise.reject({
          status: 422,
          data: { code: 'INSUFFICIENT_FUNDS', message: 'Not enough' },
        }),
    }));

    const { store, root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: baseForm,
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440003',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
          lockedAmountInput: '10.00',
          lockedDescription: 'coffee',
        },
      },
    });

    findByLabel(root, 'Send money').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().transferSubmission.status).toBe('failed');
      expect(store.getState().transferSubmission.retryable).toBe(false);
      expect(store.getState().transferSubmission.errorCode).toBe('INSUFFICIENT_FUNDS');
    });
  });

  it('blocks retry on idempotency conflict and offers new transfer', async () => {
    createTransferMock.mockImplementation(() => ({
      unwrap: () =>
        Promise.reject({
          status: 409,
          data: { code: 'IDEMPOTENCY_CONFLICT' },
        }),
    }));

    const { store, root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: baseForm,
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440004',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
          lockedAmountInput: '10.00',
          lockedDescription: 'coffee',
        },
      },
    });

    findByLabel(root, 'Send money').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().transferSubmission.errorCode).toBe('IDEMPOTENCY_CONFLICT');
      expect(findByLabel(root, 'Start new transfer')).toBeTruthy();
    });
  });

  it('maps API failures into retryable submission state', async () => {
    createTransferMock.mockImplementation(() => ({
      unwrap: () => Promise.reject({ status: 'FETCH_ERROR' }),
    }));

    const { store, root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: { ...baseForm, amountInput: '5.00', description: '' },
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440002',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
          lockedAmountInput: '5.00',
          lockedDescription: '',
        },
      },
    });

    findByLabel(root, 'Send money').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().transferSubmission.status).toBe('unknown_outcome');
      expect(store.getState().transferSubmission.retryable).toBe(true);
    });
  });

  it('ignores double-tap while submitting', async () => {
    let resolveTransfer: ((value: unknown) => void) | undefined;
    createTransferMock.mockImplementation(() => ({
      unwrap: () =>
        new Promise((resolve) => {
          resolveTransfer = resolve;
        }),
    }));

    const { root, store } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: baseForm,
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440005',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
          lockedAmountInput: '10.00',
          lockedDescription: 'coffee',
        },
      },
    });

    findByLabel(root, 'Send money').props.onPress();
    findByLabel(root, 'Send money').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().transferSubmission.status).toBe('submitting');
    });
    expect(createTransferMock).toHaveBeenCalledTimes(1);
    resolveTransfer?.({
      transfer_id: 't-dup',
      status: 'COMPLETED',
      currency: 'USD',
      sender_balance_minor: '1',
      created_at: '2025-01-01T00:00:00Z',
    });
  });
});
