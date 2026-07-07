import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

/**
 * Login form field state.
 */
export interface LoginFormFields {
  /** Username input value. */
  username: string;
  /** Password input value. */
  password: string;
}

/**
 * UI slice state for cross-screen ephemeral UI.
 */
export interface UiState {
  /** Login form field values. */
  loginForm: LoginFormFields;
  /** Global banner error message. */
  globalError: string | null;
  /** Whether the global error banner is visible. */
  showGlobalError: boolean;
}

const initialState: UiState = {
  loginForm: {
    username: "",
    password: "",
  },
  globalError: null,
  showGlobalError: false,
};

/**
 * Redux slice for UI-only ephemeral state (form fields, banners).
 */
export const uiSlice = createSlice({
  name: "ui",
  initialState,
  reducers: {
    /**
     * Updates the login username field.
     *
     * @param state Current UI state.
     * @param action Username string.
     */
    setLoginUsername(state, action: PayloadAction<string>) {
      state.loginForm.username = action.payload;
    },

    /**
     * Updates the login password field.
     *
     * @param state Current UI state.
     * @param action Password string.
     */
    setLoginPassword(state, action: PayloadAction<string>) {
      state.loginForm.password = action.payload;
    },

    /**
     * Clears login form fields.
     *
     * @param state - Current UI state.
     */
    clearLoginForm(state) {
      state.loginForm = { username: "", password: "" };
    },

    /**
     * Shows a global error banner.
     *
     * @param state Current UI state.
     * @param action Error message.
     */
    showError(state, action: PayloadAction<string>) {
      state.globalError = action.payload;
      state.showGlobalError = true;
    },

    /**
     * Dismisses the global error banner.
     *
     * @param state - Current UI state.
     */
    dismissError(state) {
      state.showGlobalError = false;
      state.globalError = null;
    },
  },
});

/** UI slice action creators. */
export const {
  setLoginUsername,
  setLoginPassword,
  clearLoginForm,
  showError,
  dismissError,
} = uiSlice.actions;

/** UI slice reducer. */
export const uiReducer = uiSlice.reducer;
