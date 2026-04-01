import { useRef, useCallback } from "react";
import { useSyncExternalStore } from "react";
import { run } from "@flickjs/runtime";

/**
 * Create a derived value that auto-tracks FlickJS signals.
 * Re-renders the component when the computed result changes.
 *
 * @example
 * ```tsx
 * import { fx } from '@flickjs/react'
 * const count = fx(0)
 *
 * function DoubleCounter() {
 *   const doubled = useComputed(() => count() * 2)
 *   return <div>{doubled}</div>
 * }
 * ```
 */
export function useComputed<T>(fn: () => T): T {
  const valueRef = useRef<{ v: T } | null>(null);
  if (valueRef.current === null) {
    valueRef.current = { v: fn() };
  }
  const fnRef = useRef(fn);
  fnRef.current = fn;

  const subscribe = useCallback((onStoreChange: () => void) => {
    const dispose = run(() => {
      valueRef.current = { v: fnRef.current() };
      onStoreChange();
    });
    return dispose;
  }, []);

  const getSnapshot = useCallback(() => valueRef.current!.v, []);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}
