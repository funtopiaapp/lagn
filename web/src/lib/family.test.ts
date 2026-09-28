import { exportJson, importJson, isMember, loadMembers, saveMembers, type Member } from "./family";

const m: Member = { id: "a1", name: "Meena", relation: "child", sex: "female", birth: { date: "2012-04-02", time: "09:15:00", latitude: 13.08, longitude: 80.27, utc_offset_hours: 5.5, place: "Chennai" } };
afterEach(() => { localStorage.clear(); vi.restoreAllMocks(); });

describe("family storage", () => {
  it("round-trips through export and import", () => {
    expect(importJson(exportJson([m]))).toEqual([m]);
  });

  it("rejects anything that is not a complete member", () => {
    expect(isMember(m)).toBe(true);
    for (const bad of [
      { ...m, relation: "cousin" },
      { ...m, birth: { ...m.birth, latitude: 123 } },
      { ...m, birth: { ...m.birth, date: "2 April 2012" } },
      { ...m, birth: { ...m.birth, utc_offset_hours: "5.5" } },
      { ...m, sex: "other" },
      { ...m, id: "" },
      { ...m, name: "x".repeat(81) },
      null, 42, "member",
    ]) expect(isMember(bad)).toBe(false);
    expect(() => importJson("not json")).toThrow(/not valid JSON/);
    expect(() => importJson(JSON.stringify({ members: [m] }))).toThrow(/not a Lagn family export/);
    expect(() => importJson(JSON.stringify({ format: "lagn-family/1", members: [m, { ...m, relation: "x" }] }))).toThrow(/Member 2/);
  });

  it("stores on the device, drops corrupt entries, and survives blocked storage", () => {
    expect(saveMembers([m])).toBe(true);
    expect(loadMembers()).toEqual([m]);
    localStorage.setItem("lagn.family.v1", JSON.stringify([m, { junk: true }]));
    expect(loadMembers()).toEqual([m]);
    localStorage.setItem("lagn.family.v1", "{broken");
    expect(loadMembers()).toEqual([]);
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("blocked"); });
    expect(saveMembers([m])).toBe(false);
  });
});
