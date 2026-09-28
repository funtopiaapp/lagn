import { readDate, readTime } from "./readback";

describe("readback", () => {
  it("reads dates unambiguously, whatever the browser locale", () => {
    expect(readDate("1985-06-21")).toBe("21 June 1985");
    expect(readDate("1985-07-06")).toBe("6 July 1985");
    expect(readDate("1400-06-15")).toBe("15 June 1400");
  });
  it("rejects malformed dates", () => {
    for (const bad of ["", "1985-13-01", "1985-00-10", "85-06-21", "1985/06/21"]) expect(readDate(bad)).toBeNull();
  });
  it("reads times on the 24-hour clock", () => {
    expect(readTime("14:30")).toBe("14:30:00 (24-hour clock)");
    expect(readTime("02:05:09")).toBe("02:05:09 (24-hour clock)");
    expect(readTime("2:30")).toBeNull();
  });
});
