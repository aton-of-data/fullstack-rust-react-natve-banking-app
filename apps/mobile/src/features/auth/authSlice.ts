import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

/**
 * Authentication status values.
 */
export type AuthStatus =
  "idle" | "loading" | "authenticated" | "unauthenticated";

/**
 * Auth slice state shape.
 */
export interface AuthState {
  /** Current auth lifecycle status. */
  status: AuthStatus;
  /** JWT access token, null when logged out. */
  accessToken: string | null;
  /** Authenticated user identifier. */
  userId: string | null;
  /** Authenticated username. */
  username: string | null;
  /** Whether SecureStore rehydration has completed. */
  hydrated: boolean;
}

const initialState: AuthState = {
  status: "idle",
  accessToken: null,
  userId: null,
  username: null,
  hydrated: false,
};

/**
 * Credentials payload for successful authentication.
 */
export interface AuthCredentials {
  /** Bearer access token. */
  accessToken: string;
  /** Authenticated user identifier. */
  userId: string;
  /** Authenticated username. */
  username: string;
}

/**
 * Redux slice managing authentication state.
 */
export const authSlice = createSlice({
  name: "auth",
  initialState,
  reducers: {
    /**
     * Marks auth rehydration as started.
     *
     * @param state - Current auth state.
     */
    rehydrateStarted(state) {
      state.status = "loading";
    },

    /**
     * Sets credentials after successful login or rehydration.
     *
     * @param state Current auth state.
     * @param action Credentials payload.
     */
    setCredentials(state, action: PayloadAction<AuthCredentials>) {
      state.accessToken = action.payload.accessToken;
      state.userId = action.payload.userId;
      state.username = action.payload.username;
      state.status = "authenticated";
      state.hydrated = true;
    },

    /**
     * Marks rehydration complete with no stored credentials.
     *
     * @param state - Current auth state.
     */
    rehydrateEmpty(state) {
      state.status = "unauthenticated";
      state.hydrated = true;
    },

    /**
     * Clears all auth state on logout.
     *
     * @param state - Current auth state.
     */
    clearCredentials(state) {
      state.accessToken = null;
      state.userId = null;
      state.username = null;
      state.status = "unauthenticated";
    },
  },
});

/** Auth slice action creators. */
export const {
  rehydrateStarted,
  setCredentials,
  rehydrateEmpty,
  clearCredentials,
} = authSlice.actions;

/** Auth slice reducer. */
export const authReducer = authSlice.reducer;
