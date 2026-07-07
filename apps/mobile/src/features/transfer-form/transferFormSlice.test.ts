import { describe, expect, it } from 'vitest';

import {
  selectRecipient,
  setAmountInput,
  setSearchQuery,
  transferFormReducer,
  goToConfirm,
  resetForm,
} from './transferFormSlice';

describe('transferFormReducer', () => {
  it('updates search query', () => {
    const state = transferFormReducer(undefined, setSearchQuery('ali'));
    expect(state.searchQuery).toBe('ali');
  });

  it('selects recipient and advances to form step', () => {
    const state = transferFormReducer(
      undefined,
      selectRecipient({ userId: 'id-1', username: 'alice' }),
    );

    expect(state.step).toBe('form');
    expect(state.recipientUsername).toBe('alice');
  });

  it('advances to confirm when amount is set', () => {
    let state = transferFormReducer(
      undefined,
      selectRecipient({ userId: 'id-1', username: 'alice' }),
    );
    state = transferFormReducer(state, setAmountInput('10.00'));
    state = transferFormReducer(state, goToConfirm());

    expect(state.step).toBe('confirm');
    expect(state.amountInput).toBe('10.00');
  });

  it('resets to initial state', () => {
    let state = transferFormReducer(
      undefined,
      selectRecipient({ userId: 'id-1', username: 'alice' }),
    );
    state = transferFormReducer(state, resetForm());

    expect(state.step).toBe('search');
    expect(state.recipientUsername).toBeNull();
  });
});
