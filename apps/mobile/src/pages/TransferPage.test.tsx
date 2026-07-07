import { describe, expect, it, vi } from 'vitest';

import { TransferPage } from './TransferPage';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

vi.mock('@/services', async () => {
  const actual = await vi.importActual<typeof import('@/services')>('@/services');
  return {
    ...actual,
    useSearchUsersQuery: () => ({
      data: { items: [] },
      isLoading: false,
      isFetching: false,
      isError: false,
      refetch: vi.fn(),
    }),
    useCreateTransferMutation: () => [vi.fn(), { isLoading: false }],
  };
});

describe('TransferPage', () => {
  it('renders recipient search on the first wizard step', () => {
    const { root } = renderTestTree(<TransferPage />);
    expect(findByText(root, 'Send Money')).toBeTruthy();
    expect(findByLabel(root, 'Search recipient username')).toBeTruthy();
  });

  it('renders confirmation step when form state is confirm', () => {
    const { root } = renderTestTree(<TransferPage />, {
      preloadedState: {
        transferForm: {
          step: 'confirm',
          searchQuery: '',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '10.00',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
        transferSubmission: {
          idempotencyKey: '550e8400-e29b-41d4-a716-446655440000',
          status: 'idle',
          lastTransferId: null,
          retryable: false,
          errorCode: null,
          errorMessage: null,
        },
      },
    });

    expect(findByText(root, 'Confirm transfer')).toBeTruthy();
    expect(findByText(root, '@bob')).toBeTruthy();
  });
});
