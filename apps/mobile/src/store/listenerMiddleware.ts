import { createListenerMiddleware } from "@reduxjs/toolkit";
import * as SecureStore from "expo-secure-store";

import {
  authSlice,
  clearCredentials,
  rehydrateEmpty,
  rehydrateStarted,
  setCredentials,
} from "@/features/auth";

/** SecureStore key for the JWT access token. */
export const ACCESS_TOKEN_KEY = "ficus_access_token";

/** SecureStore key for the authenticated user id. */
export const USER_ID_KEY = "ficus_user_id";

/** SecureStore key for the authenticated username. */
export const USERNAME_KEY = "ficus_username";

/**
 * Listener middleware for auth persistence via Expo SecureStore.
 */
export const listenerMiddleware = createListenerMiddleware();

/**
 * Rehydrates auth state from SecureStore on app launch.
 */
listenerMiddleware.startListening({
  actionCreator: rehydrateStarted,
  effect: async (_action, listenerApi) => {
    try {
      const [token, userId, username] = await Promise.all([
        SecureStore.getItemAsync(ACCESS_TOKEN_KEY),
        SecureStore.getItemAsync(USER_ID_KEY),
        SecureStore.getItemAsync(USERNAME_KEY),
      ]);

      if (token && userId && username) {
        listenerApi.dispatch(
          setCredentials({ accessToken: token, userId, username }),
        );
      } else {
        listenerApi.dispatch(rehydrateEmpty());
      }
    } catch {
      listenerApi.dispatch(rehydrateEmpty());
    }
  },
});

/**
 * Persists credentials to SecureStore after login or rehydration.
 */
listenerMiddleware.startListening({
  actionCreator: setCredentials,
  effect: async (action) => {
    await Promise.all([
      SecureStore.setItemAsync(ACCESS_TOKEN_KEY, action.payload.accessToken),
      SecureStore.setItemAsync(USER_ID_KEY, action.payload.userId),
      SecureStore.setItemAsync(USERNAME_KEY, action.payload.username),
    ]);
  },
});

/**
 * Clears persisted credentials on logout.
 */
listenerMiddleware.startListening({
  actionCreator: clearCredentials,
  effect: async () => {
    await Promise.all([
      SecureStore.deleteItemAsync(ACCESS_TOKEN_KEY),
      SecureStore.deleteItemAsync(USER_ID_KEY),
      SecureStore.deleteItemAsync(USERNAME_KEY),
    ]);
  },
});

/**
 * Dispatches rehydrate on store init.
 *
 * @returns Rehydrate started action.
 */
export function bootstrapAuth(): ReturnType<typeof rehydrateStarted> {
  return rehydrateStarted();
}

/** Auth slice reference for listener registration. */
export { authSlice };
