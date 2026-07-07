import { describe, expect, it } from 'vitest';

import { dismissError, showError, uiReducer } from './uiSlice';

describe('uiReducer', () => {
  it('shows global error banner', () => {
    const state = uiReducer(undefined, showError('Something went wrong'));

    expect(state.showGlobalError).toBe(true);
    expect(state.globalError).toBe('Something went wrong');
  });

  it('dismisses global error banner', () => {
    const withError = uiReducer(undefined, showError('Error'));
    const state = uiReducer(withError, dismissError());

    expect(state.showGlobalError).toBe(false);
    expect(state.globalError).toBeNull();
  });
});
