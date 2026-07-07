import type { RootState } from "@/store";

/**
 * Selects login form field values.
 *
 * @param state Root Redux state.
 * @returns Login form username and password.
 */
export function selectLoginForm(state: RootState): {
  username: string;
  password: string;
} {
  return state.ui.loginForm;
}

/**
 * Selects the global error message.
 *
 * @param state Root Redux state.
 * @returns Error message or null.
 */
export function selectGlobalError(state: RootState): string | null {
  return state.ui.globalError;
}

/**
 * Selects whether the global error banner is visible.
 *
 * @param state Root Redux state.
 * @returns True when visible.
 */
export function selectShowGlobalError(state: RootState): boolean {
  return state.ui.showGlobalError;
}
