import { useRef } from "react";
import { aiObject } from "@flickjs/ai";
import type { AiObjectOptions } from "@flickjs/ai";
import { useFxValue } from "../internal/use-fx-value";

export interface UseAiObjectReturn<T> {
  object: Partial<T> | undefined;
  error: Error | undefined;
  isLoading: boolean;
  submit: (input: string | Record<string, unknown>) => Promise<void>;
  stop: () => void;
}

/**
 * React hook for streaming structured AI objects with schema validation.
 *
 * @example
 * ```tsx
 * import { useAiObject } from '@flickjs/react/ai'
 * import { z } from 'zod'
 *
 * const schema = z.object({
 *   name: z.string(),
 *   ingredients: z.array(z.string()),
 * })
 *
 * function RecipeGenerator() {
 *   const { object, isLoading, submit } = useAiObject({
 *     api: '/api/recipe',
 *     schema,
 *   })
 *
 *   return (
 *     <div>
 *       <button onClick={() => submit('healthy breakfast')}>Generate</button>
 *       {isLoading && <p>Generating...</p>}
 *       {object && <pre>{JSON.stringify(object, null, 2)}</pre>}
 *     </div>
 *   )
 * }
 * ```
 */
export function useAiObject<T>(
  options: Omit<AiObjectOptions<T>, "suspense">
): UseAiObjectReturn<T> {
  const objRef = useRef<ReturnType<typeof aiObject<T>> | null>(null);
  if (objRef.current === null) {
    objRef.current = aiObject<T>({ ...options, suspense: false });
  }
  const obj = objRef.current;

  const object = useFxValue(obj.object);
  const error = useFxValue(obj.error);
  const loading = useFxValue(obj.isLoading);

  return {
    object,
    error,
    isLoading: loading,
    submit: obj.submit,
    stop: obj.stop,
  };
}
