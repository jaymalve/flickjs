import { useEffect, useRef } from "react";
import { run } from "@flickjs/runtime";

/**
 * Run a side effect that auto-tracks FlickJS signals.
 * Re-runs when any tracked signal changes. Cleans up on unmount.
 *
 * @example
 * ```tsx
 * import { fx } from '@flickjs/react'
 * const count = fx(0)
 *
 * function Logger() {
 *   useFxEffect(() => {
 *     console.log('Count changed:', count())
 *   })
 *   return null
 * }
 * ```
 */
export function useFxEffect(fn: () => void): void {
  const fnRef = useRef(fn);
  fnRef.current = fn;

  useEffect(() => {
    const dispose = run(() => {
      fnRef.current();
    });
    return dispose;
  }, []);
}
