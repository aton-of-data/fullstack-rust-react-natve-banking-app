import type { RootState } from '@/store';

/**
 * Selects the current authentication status.
 *
 * @param state Root Redux state.
 * @returns Auth status string.
 */
export function selectAuthStatus(state: RootState): string {
  return state.auth.status;
}

/**
 * Selects whether the user is authenticated.
 *
 * @param state Root Redux state.
 * @returns True when authenticated.
 */
export function selectIsAuthenticated(state: RootState): boolean {
  return state.auth.status === 'authenticated';
}

/**
 * Selects the access token.
 *
 * @param state Root Redux state.
 * @returns JWT access token or null.
 */
export function selectAccessToken(state: RootState): string | null {
  return state.auth.accessToken;
}

/**
 * Selects the authenticated username.
 *
 * @param state Root Redux state.
 * @returns Username or null.
 */
export function selectUsername(state: RootState): string | null {
  return state.auth.username;
}

/**
 * Selects whether SecureStore rehydration has completed.
 *
 * @param state Root Redux state.
 * @returns True when hydrated.
 */
export function selectAuthHydrated(state: RootState): boolean {
  return state.auth.hydrated;
}
