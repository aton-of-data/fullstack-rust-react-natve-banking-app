import { describe, expect, it } from 'vitest';

import {
  selectAmountInput,
  selectDescription,
  selectRecipientUsername,
  selectSearchQuery,
  selectTransferStep,
  selectTransferSubmitError,
  selectTransferSubmitting,
} from './selectors';
import {
  goToConfirm,
  selectRecipient,
  setAmountInput,
  setDescription,
  setSearchQuery,
  submitFailed,
  submitStarted,
} from './transferFormSlice';
import { createTestStore } from '@/test/renderWithProviders';

describe('transfer form selectors', () => {
  it('tracks wizard progression', () => {
    const store = createTestStore();
    store.dispatch(setSearchQuery('bo'));
    store.dispatch(selectRecipient({ userId: 'u2', username: 'bob' }));
    store.dispatch(setAmountInput('12.50'));
    store.dispatch(setDescription('lunch'));
    store.dispatch(goToConfirm());

    const state = store.getState();
    expect(selectTransferStep(state)).toBe('confirm');
    expect(selectSearchQuery(state)).toBe('bo');
    expect(selectRecipientUsername(state)).toBe('bob');
    expect(selectAmountInput(state)).toBe('12.50');
    expect(selectDescription(state)).toBe('lunch');
  });

  it('tracks submission state', () => {
    const store = createTestStore();
    store.dispatch(submitStarted());
    expect(selectTransferSubmitting(store.getState())).toBe(true);
    store.dispatch(submitFailed('timeout'));
    expect(selectTransferSubmitError(store.getState())).toBe('timeout');
  });
});
