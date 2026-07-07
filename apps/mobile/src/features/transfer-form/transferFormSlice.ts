import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

/**
 * Transfer wizard step identifiers.
 */
export type TransferStep = "search" | "form" | "confirm";

/**
 * Transfer form slice state shape.
 */
export interface TransferFormState {
  /** Current wizard step. */
  step: TransferStep;
  /** Recipient search query text. */
  searchQuery: string;
  /** Selected recipient user identifier. */
  recipientUserId: string | null;
  /** Selected recipient username. */
  recipientUsername: string | null;
  /** Amount input as major-unit decimal string (e.g. "12.34"). */
  amountInput: string;
  /** Optional transfer memo. */
  description: string;
  /** ISO 4217 currency code. */
  currency: string;
  /** Submission in progress flag. */
  submitting: boolean;
  /** Last submission error message. */
  submitError: string | null;
}

const initialState: TransferFormState = {
  step: "search",
  searchQuery: "",
  recipientUserId: null,
  recipientUsername: null,
  amountInput: "",
  description: "",
  currency: "USD",
  submitting: false,
  submitError: null,
};

/**
 * Redux slice managing the transfer wizard form state.
 */
export const transferFormSlice = createSlice({
  name: "transferForm",
  initialState,
  reducers: {
    /**
     * Updates the recipient search query.
     *
     * @param state Current form state.
     * @param action Search query string.
     */
    setSearchQuery(state, action: PayloadAction<string>) {
      state.searchQuery = action.payload;
    },

    /**
     * Selects a recipient and advances to the amount form step.
     *
     * @param state Current form state.
     * @param action Recipient user id and username.
     */
    selectRecipient(
      state,
      action: PayloadAction<{ userId: string; username: string }>,
    ) {
      state.recipientUserId = action.payload.userId;
      state.recipientUsername = action.payload.username;
      state.step = "form";
      state.submitError = null;
    },

    /**
     * Updates the amount input field.
     *
     * @param state Current form state.
     * @param action Amount string.
     */
    setAmountInput(state, action: PayloadAction<string>) {
      state.amountInput = action.payload;
    },

    /**
     * Updates the transfer description.
     *
     * @param state Current form state.
     * @param action Description string.
     */
    setDescription(state, action: PayloadAction<string>) {
      state.description = action.payload;
    },

    /**
     * Advances to the confirmation step.
     *
     * @param state - Current form state.
     */
    goToConfirm(state) {
      state.step = "confirm";
      state.submitError = null;
    },

    /**
     * Returns to the amount form step from confirmation.
     *
     * @param state - Current form state.
     */
    backToForm(state) {
      state.step = "form";
    },

    /**
     * Returns to recipient search step.
     *
     * @param state - Current form state.
     */
    backToSearch(state) {
      state.step = "search";
      state.recipientUserId = null;
      state.recipientUsername = null;
    },

    /**
     * Marks transfer submission as in progress.
     *
     * @param state - Current form state.
     */
    submitStarted(state) {
      state.submitting = true;
      state.submitError = null;
    },

    /**
     * Marks transfer submission as failed.
     *
     * @param state Current form state.
     * @param action Error message.
     */
    submitFailed(state, action: PayloadAction<string>) {
      state.submitting = false;
      state.submitError = action.payload;
    },

    /**
     * Resets the form after successful transfer.
     *
     * @returns Initial form state.
     */
    submitSucceeded() {
      return { ...initialState };
    },

    /**
     * Fully resets the transfer form.
     *
     * @returns Initial form state.
     */
    resetForm() {
      return initialState;
    },
  },
});

/** Transfer form action creators. */
export const {
  setSearchQuery,
  selectRecipient,
  setAmountInput,
  setDescription,
  goToConfirm,
  backToForm,
  backToSearch,
  submitStarted,
  submitFailed,
  submitSucceeded,
  resetForm,
} = transferFormSlice.actions;

/** Transfer form reducer. */
export const transferFormReducer = transferFormSlice.reducer;
