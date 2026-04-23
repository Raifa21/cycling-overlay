import { describe, it, expect } from "vitest";
import { parseTimeSpec, formatTimeSpec } from "./time";

describe("parseTimeSpec", () => {
  it("parses bare seconds", () => {
    expect(parseTimeSpec("0")).toBe(0);
    expect(parseTimeSpec("90")).toBe(90);
    expect(parseTimeSpec("90.5")).toBe(90.5);
  });

  it("parses MM:SS", () => {
    expect(parseTimeSpec("01:30")).toBe(90);
    expect(parseTimeSpec("2:30")).toBe(150);
    expect(parseTimeSpec("1:30.25")).toBe(90.25);
  });

  it("parses HH:MM:SS", () => {
    expect(parseTimeSpec("01:23:45")).toBe(5025);
    expect(parseTimeSpec("00:00:00")).toBe(0);
    expect(parseTimeSpec("1:02:03.5")).toBe(3723.5);
  });

  it("rejects empty and malformed", () => {
    expect(parseTimeSpec("")).toBeNull();
    expect(parseTimeSpec("   ")).toBeNull();
    expect(parseTimeSpec("abc")).toBeNull();
    expect(parseTimeSpec("1:2:3:4")).toBeNull();
  });

  it("rejects out-of-range components", () => {
    expect(parseTimeSpec("1:60")).toBeNull(); // seconds must be < 60
    expect(parseTimeSpec("1:90")).toBeNull();
    expect(parseTimeSpec("1:60:00")).toBeNull(); // minutes must be < 60
    expect(parseTimeSpec("-1")).toBeNull();
  });
});

describe("formatTimeSpec", () => {
  it("pads whole seconds to HH:MM:SS", () => {
    expect(formatTimeSpec(0)).toBe("00:00:00");
    expect(formatTimeSpec(65)).toBe("00:01:05");
    expect(formatTimeSpec(3661)).toBe("01:01:01");
  });

  it("shows fractional seconds with ms precision", () => {
    expect(formatTimeSpec(3.7)).toBe("00:00:03.700");
    expect(formatTimeSpec(65.25)).toBe("00:01:05.250");
  });

  it("clamps negative and non-finite to zero", () => {
    expect(formatTimeSpec(-1)).toBe("00:00:00");
    expect(formatTimeSpec(NaN)).toBe("00:00:00");
    expect(formatTimeSpec(Infinity)).toBe("00:00:00");
  });

  it("roundtrips with parseTimeSpec on whole seconds", () => {
    for (const s of [0, 1, 59, 60, 3599, 3600, 5025]) {
      expect(parseTimeSpec(formatTimeSpec(s))).toBe(s);
    }
  });
});
