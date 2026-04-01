import type { Fx } from "@flickjs/runtime";
import { useFxValue } from "../internal/use-fx-value";

/**
 * Subscribe a React component to a FlickJS signal.
 * The component re-renders when the signal value changes.
 *
 * @example
 * ```tsx
 * import { fx } from '@flickjs/react'
 * const count = fx(0)
 *
 * function Counter() {
 *   const value = useFx(count)
 *   return <div>{value}</div>
 * }
 * ```
 */
export function useFx<T>(signal: Fx<T>): T {
  return useFxValue(signal);
}
