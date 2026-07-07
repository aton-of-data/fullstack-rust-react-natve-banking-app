import { describe, expect, it } from 'vitest';

import { TransferAmountForm } from './TransferAmountForm';
import { findByLabel, findByText, renderTestTree } from '@/test/renderTestTree';

describe('TransferAmountForm', () => {
  it('advances to confirm and generates idempotency key on review', () => {
    const { store, root } = renderTestTree(<TransferAmountForm />, {
      preloadedState: {
        transferForm: {
          step: 'form',
          searchQuery: '',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
      },
    });

    findByLabel(root, 'Transfer amount').props.onChangeText('12.50');
    findByLabel(root, 'Review').props.onPress();

    const state = store.getState();
    expect(state.transferForm.step).toBe('confirm');
    expect(state.transferSubmission.idempotencyKey).toBeTruthy();
    expect(findByText(root, 'Send to @bob')).toBeTruthy();
  });

  it('shows validation hint for invalid amounts', () => {
    const { root } = renderTestTree(<TransferAmountForm />, {
      preloadedState: {
        transferForm: {
          step: 'form',
          searchQuery: '',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '0',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
      },
    });

    expect(findByText(root, 'Enter a valid amount greater than zero')).toBeTruthy();
  });

  it('returns to recipient search from the amount step', () => {
    const { store, root } = renderTestTree(<TransferAmountForm />, {
      preloadedState: {
        transferForm: {
          step: 'form',
          searchQuery: 'bo',
          recipientUserId: 'u2',
          recipientUsername: 'bob',
          amountInput: '5.00',
          description: '',
          currency: 'USD',
          submitting: false,
          submitError: null,
        },
      },
    });

    findByLabel(root, 'Back').props.onPress();
    expect(store.getState().transferForm.step).toBe('search');
  });
});
