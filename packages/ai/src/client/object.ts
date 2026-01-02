import { fx, getCurrentSuspense } from "@flickjs/runtime";
import type { Fx } from "@flickjs/runtime";
import type { AiObject, AiObjectOptions } from "./types";
import { parseStream } from "../utils/stream-parser";

/**
 * Create a reactive AI object generator
 *
 * @example
 * ```tsx
 * import { aiObject } from "@flickjs/ai";
 * import { z } from "zod";
 *
 * const recipe = aiObject({
 *   api: '/api/recipe',
 *   schema: z.object({
 *     name: z.string(),
 *     ingredients: z.array(z.string()),
 *     steps: z.array(z.string())
 *   })
 * });
 *
 * // Reactive state
 * recipe.object()     // Partial<Recipe> | undefined
 * recipe.isLoading()  // boolean
 * recipe.error()      // Error | undefined
 *
 * // Submit to generate
 * recipe.submit("A healthy breakfast recipe")
 * ```
 */
export function aiObject<T>(options: AiObjectOptions<T>): AiObject<T> {
  const {
    api,
    schema,
    headers,
    body,
    onFinish,
    onError,
    suspense = false,
    credentials,
  } = options;

  // Reactive state
  const object: Fx<Partial<T> | undefined> = fx<Partial<T> | undefined>(
    undefined
  );
  const error: Fx<Error | undefined> = fx<Error | undefined>(undefined);
  const loading = fx<boolean>(false);

  // Track current abort controller for cancellation
  let abortController: AbortController | null = null;

  /**
   * Derived loading state
   */
  const isLoading = (): boolean => loading();

  /**
   * Submit input to generate the object
   */
  const submit = async (input: string | Record<string, unknown>): Promise<void> => {
    // Reset state
    object.set(undefined);
    error.set(undefined);
    loading.set(true);

    // Create abort controller
    abortController = new AbortController();

    // Create promise for Suspense integration
    const streamPromise = (async () => {
      try {
        const response = await fetch(api, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            ...headers,
          },
          body: JSON.stringify({
            input: typeof input === "string" ? input : undefined,
            ...(typeof input === "object" ? input : {}),
            ...body,
          }),
          signal: abortController!.signal,
          credentials,
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        if (!response.body) {
          throw new Error("Response body is empty");
        }

        const reader = response.body.getReader();

        // Parse the stream - AI SDK sends partial objects directly
        for await (const part of parseStream(reader)) {
          if (part.type === "object") {
            object.set(part.value as Partial<T>);
          } else if (part.type === "error") {
            throw new Error(String(part.value));
          }
        }

        // Validate final object with schema
        loading.set(false);
        abortController = null;

        const finalObject = object();
        if (finalObject) {
          try {
            const validated = schema.parse(finalObject) as T;
            object.set(validated);
            onFinish?.(validated);
          } catch (validationError) {
            // Keep the partial object but report validation error
            const err =
              validationError instanceof Error
                ? validationError
                : new Error(String(validationError));
            error.set(err);
            onError?.(err);
          }
        }
      } catch (err) {
        // Handle abort
        if (err instanceof Error && err.name === "AbortError") {
          loading.set(false);
          return;
        }

        const errorInstance =
          err instanceof Error ? err : new Error(String(err));
        error.set(errorInstance);
        loading.set(false);
        onError?.(errorInstance);
      }
    })();

    // Register with Suspense if enabled
    if (suspense) {
      const suspenseContext = getCurrentSuspense();
      if (suspenseContext) {
        suspenseContext.register(streamPromise);
      }
    }

    await streamPromise;
  };

  /**
   * Stop the current stream
   */
  const stop = (): void => {
    if (abortController) {
      abortController.abort();
      abortController = null;
      loading.set(false);
    }
  };

  return {
    object,
    error,
    isLoading,
    submit,
    stop,
  };
}
