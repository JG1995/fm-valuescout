import { useCallback, useEffect, useRef, useState } from "react";

/** Long enough to notice the tick, short enough not to hide the label. */
const HOLD_MS = 1400;

/**
 * Drives a button's `done` tick: call `flash` when the action has worked, and
 * `done` is true for a moment.
 */
export function useDoneFlash() {
  const [done, setDone] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const flash = useCallback(() => {
    window.clearTimeout(timer.current);
    setDone(true);
    timer.current = window.setTimeout(() => setDone(false), HOLD_MS);
  }, []);

  return [done, flash] as const;
}
