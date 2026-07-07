import { describe, expect, it } from "vitest";

import {
  addMinor,
  formatMinorUnits,
  MoneyError,
  parseMinorUnits,
  subtractMinor,
} from "./index.js";

describe("@ficus/money", () => {
  it("parses and normalizes minor units", () => {
    expect(parseMinorUnits("00123")).toBe("123");
  });

  it("rejects fractional minor units", () => {
    expect(() => parseMinorUnits("12.34")).toThrow(MoneyError);
  });

  it("adds and subtracts with bigint arithmetic", () => {
    expect(addMinor("100", "50")).toBe("150");
    expect(subtractMinor("100", "50")).toBe("50");
  });

  it("formats minor units for display", () => {
    expect(formatMinorUnits("1234", { currency: "USD" })).toBe("$12.34");
  });
});
