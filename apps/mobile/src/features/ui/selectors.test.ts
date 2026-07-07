import { describe, expect, it } from 'vitest';

import { selectGlobalError, selectLoginForm, selectShowGlobalError } from './selectors';
import { setLoginPassword, setLoginUsername, showError } from './uiSlice';
import { createTestStore } from '@/test/renderWithProviders';

describe('ui selectors', () => {
  it('selects login form values', () => {
    const store = createTestStore();
    store.dispatch(setLoginUsername('alice'));
    store.dispatch(setLoginPassword('secret'));
    expect(selectLoginForm(store.getState())).toEqual({
      username: 'alice',
      password: 'secret',
    });
  });

  it('selects global error visibility', () => {
    const store = createTestStore();
    store.dispatch(showError('Network down'));
    const state = store.getState();
    expect(selectGlobalError(state)).toBe('Network down');
    expect(selectShowGlobalError(state)).toBe(true);
  });
});
