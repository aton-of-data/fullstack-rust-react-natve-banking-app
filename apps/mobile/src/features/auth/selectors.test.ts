import { describe, expect, it } from 'vitest';

import {
  selectAccessToken,
  selectAuthHydrated,
  selectAuthStatus,
  selectIsAuthenticated,
  selectUsername,
} from './selectors';
import { setCredentials } from './authSlice';
import { createTestStore } from '@/test/renderWithProviders';

describe('auth selectors', () => {
  it('reflects authenticated credentials', () => {
    const store = createTestStore();
    store.dispatch(setCredentials({ accessToken: 'token', userId: 'user-1', username: 'alice' }));
    const state = store.getState();
    expect(selectAuthStatus(state)).toBe('authenticated');
    expect(selectIsAuthenticated(state)).toBe(true);
    expect(selectAccessToken(state)).toBe('token');
    expect(selectUsername(state)).toBe('alice');
    expect(selectAuthHydrated(state)).toBe(true);
  });

  it('returns defaults for anonymous state', () => {
    const state = createTestStore().getState();
    expect(selectIsAuthenticated(state)).toBe(false);
    expect(selectAccessToken(state)).toBeNull();
    expect(selectUsername(state)).toBeNull();
  });
});
