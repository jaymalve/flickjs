import { useCallback } from "react";
import { useSyncExternalStore } from "react";
import { run } from "@flickjs/runtime";

/**
 * Internal helper: subscribe to any fx-compatible getter via useSyncExternalStore.
 * Uses run() to auto-track signal dependencies and dispose() to clean up on unmount.
 */
export function useFxValue<T>(signal: () => T): T {
  const subscribe = useCallback(
    (onStoreChange: () => void) => {
      const dispose = run(() => {
        signal();
        onStoreChange();
      });
      return dispose;
    },
    [signal]
  );

  const getSnapshot = useCallback(() => signal(), [signal]);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}
