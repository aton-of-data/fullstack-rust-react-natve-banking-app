import type { RootState } from '@/store';

/**
 * Selects the current transfer wizard step.
 *
 * @param state Root Redux state.
 * @returns Transfer step identifier.
 */
export function selectTransferStep(state: RootState): string {
  return state.transferForm.step;
}

/**
 * Selects the recipient search query.
 *
 * @param state Root Redux state.
 * @returns Search query string.
 */
export function selectSearchQuery(state: RootState): string {
  return state.transferForm.searchQuery;
}

/**
 * Selects the selected recipient username.
 *
 * @param state Root Redux state.
 * @returns Recipient username or null.
 */
export function selectRecipientUsername(state: RootState): string | null {
  return state.transferForm.recipientUsername;
}

/**
 * Selects the amount input value.
 *
 * @param state Root Redux state.
 * @returns Amount input string.
 */
export function selectAmountInput(state: RootState): string {
  return state.transferForm.amountInput;
}

/**
 * Selects the transfer description.
 *
 * @param state Root Redux state.
 * @returns Description string.
 */
export function selectDescription(state: RootState): string {
  return state.transferForm.description;
}

/**
 * Selects whether a transfer submission is in progress.
 *
 * @param state Root Redux state.
 * @returns True when submitting.
 */
export function selectTransferSubmitting(state: RootState): boolean {
  return state.transferForm.submitting;
}

/**
 * Selects the last transfer submission error.
 *
 * @param state Root Redux state.
 * @returns Error message or null.
 */
export function selectTransferSubmitError(state: RootState): string | null {
  return state.transferForm.submitError;
}
