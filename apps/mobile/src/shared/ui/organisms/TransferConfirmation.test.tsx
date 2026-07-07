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
        transferForm: {
          step: 'confirm',
          searchQuery: '',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '10.00',
          description: 'coffee',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
        transferSubmission: {
          idempotencyKey,
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
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

  it('shows retry guidance for unknown outcomes', () => {
    const { root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: {
          step: 'confirm',
          searchQuery: '',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '5.00',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440001',
          status: 'unknown_outcome',
          lastTransferId: null,
          retryable: true,
          errorCode: 'NETWORK_ERROR',
          errorMessage: 'Request timed out',
        },
      },
    });

    expect(findByText(root, 'Request timed out')).toBeTruthy();
    expect(findByLabel(root, 'Retry transfer')).toBeTruthy();
  });

  it('maps API failures into retryable submission state', async () => {
    createTransferMock.mockImplementation(() => ({
      unwrap: () => Promise.reject({ status: 'FETCH_ERROR' }),
    }));

    const { store, root } = renderTestTree(<TransferConfirmation />, {
      preloadedState: {
        transferForm: {
          step: 'confirm',
          searchQuery: '',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '5.00',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440002',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
        },
      },
    });

    findByLabel(root, 'Send money').props.onPress();
    await vi.waitFor(() => {
      expect(store.getState().transferSubmission.status).toBe('unknown_outcome');
      expect(store.getState().transferSubmission.retryable).toBe(true);
    });
  });
});
