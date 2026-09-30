import { beginTransportDecision, transportReady } from "./transport";

describe("the transport decision", () => {
  it("resolves at once when nothing is pending", async () => {
    await expect(Promise.race([transportReady(), Promise.reject(new Error("blocked"))])).resolves.toBeUndefined();
  });

  it("makes callers wait until the engine has been looked for", async () => {
    const done = beginTransportDecision();
    let settled = false;
    const waiter = transportReady().then(() => { settled = true; });
    await Promise.resolve();
    expect(settled).toBe(false); // a reading made now must not fall through to HTTP
    done();
    await waiter;
    expect(settled).toBe(true);
  });

  it("releases callers even when the engine fails to load", async () => {
    const done = beginTransportDecision();
    const waiter = transportReady();
    done(); // finally() runs on failure too
    await expect(waiter).resolves.toBeUndefined();
  });
});
