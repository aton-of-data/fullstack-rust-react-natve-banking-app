import { describe, expect, it } from "vitest";

import {
  authReducer,
  clearCredentials,
  rehydrateEmpty,
  setCredentials,
} from "./authSlice";

describe("authReducer", () => {
  it("sets credentials on login", () => {
    const state = authReducer(
      undefined,
      setCredentials({
        accessToken: "token",
        userId: "user-1",
        username: "alice",
      }),
    );

    expect(state.status).toBe("authenticated");
    expect(state.accessToken).toBe("token");
    expect(state.username).toBe("alice");
    expect(state.hydrated).toBe(true);
  });

  it("clears credentials on logout", () => {
    const loggedIn = authReducer(
      undefined,
      setCredentials({ accessToken: "t", userId: "u", username: "bob" }),
    );
    const state = authReducer(loggedIn, clearCredentials());

    expect(state.status).toBe("unauthenticated");
    expect(state.accessToken).toBeNull();
  });

  it("marks unauthenticated when rehydration finds no token", () => {
    const state = authReducer(undefined, rehydrateEmpty());

    expect(state.status).toBe("unauthenticated");
    expect(state.hydrated).toBe(true);
  });
});
