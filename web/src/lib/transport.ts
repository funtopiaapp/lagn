// Which transport answers a reading is decided at start-up, and deciding it
// can take a moment: the WebAssembly engine has to be fetched before it can
// say whether it is there.
//
// The UI renders immediately regardless — but a call made in that first moment
// must wait for the answer, or it would fall through to HTTP and, on a static
// deployment where no server exists, fail. Nothing else in the app knows about
// this: `api.ts` awaits `transportReady` before choosing.

/** Resolved unless a decision is in progress. */
let ready: Promise<void> = Promise.resolve();

/**
 * Declare that the transport is being decided. Returns the function to call
 * when it is settled, whatever the outcome.
 */
export function beginTransportDecision(): () => void {
  let done!: () => void;
  ready = new Promise<void>((resolve) => {
    done = resolve;
  });
  return () => done();
}

/** Await the decision. Resolves at once when none is pending. */
export function transportReady(): Promise<void> {
  return ready;
}
