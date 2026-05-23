import { fx, getCurrentSuspense } from '@flickjs/runtime';
import type { AiObject, AiObjectOptions } from './types';
import { parseStream } from '../utils/stream-parser';
import {
  createAiClientLogger,
  summarizeBody,
  summarizeHeaders,
  summarizeJsonPayload
} from './logger';

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
  const { api, schema, headers, body, onFinish, onError, suspense = false, credentials } = options;
  const logger = createAiClientLogger('object');
  let requestCount = 0;

  // Reactive state
  const object = fx<Partial<T> | undefined>(undefined);
  const error = fx<Error | undefined>(undefined);
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
    const requestId = ++requestCount;
    const startedAt = Date.now();
    const inputType = typeof input === 'string' ? 'string' : 'object';
    const rawInputKeyCount =
      input !== null && typeof input === 'object' ? Object.keys(input).length : undefined;

    logger.debug('submit:start', {
      requestId,
      api,
      inputType,
      rawInputLength: typeof input === 'string' ? input.length : undefined,
      rawInputKeyCount,
      hasCredentials: credentials !== undefined
    });

    // Reset state
    object.set(undefined);
    error.set(undefined);
    loading.set(true);

    // Create abort controller
    abortController = new AbortController();
    logger.debug('submit:abort-controller-created', {
      requestId,
      api
    });

    // Create promise for Suspense integration
    const streamPromise = (async () => {
      try {
        const requestBody = JSON.stringify({
          input: typeof input === 'string' ? input : undefined,
          ...(typeof input === 'object' ? input : {}),
          ...body
        });

        logger.debug('submit:normalized', {
          requestId,
          api,
          inputType,
          suspense,
          ...summarizeHeaders(headers),
          ...summarizeBody(body),
          ...summarizeJsonPayload(requestBody)
        });

        const response = await fetch(api, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            ...headers
          },
          body: requestBody,
          signal: abortController!.signal,
          credentials
        });

        logger.debug('submit:response', {
          requestId,
          api,
          status: response.status,
          ok: response.ok,
          hasBody: response.body !== null,
          contentType: response.headers.get('content-type'),
          contentLength: response.headers.get('content-length')
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        if (!response.body) {
          throw new Error('Response body is empty');
        }

        const reader = response.body.getReader();

        logger.debug('submit:stream-start', {
          requestId,
          api
        });

        // Parse the stream - AI SDK sends partial objects directly
        for await (const part of parseStream(reader)) {
          if (part.type === 'object') {
            object.set(part.value as Partial<T>);
          } else if (part.type === 'error') {
            throw new Error(String(part.value));
          }
        }

        logger.debug('submit:stream-end', {
          requestId,
          api
        });

        // Validate final object with schema
        loading.set(false);
        abortController = null;

        const finalObject = object();
        if (finalObject) {
          try {
            logger.debug('submit:schema-parse-start', {
              requestId,
              api,
              finalObjectKeyCount: Object.keys(finalObject as Record<string, unknown>).length
            });
            const validated = schema.parse(finalObject) as T;
            object.set(validated);
            logger.debug('submit:success', {
              requestId,
              api,
              durationMs: Date.now() - startedAt,
              finalObjectKeyCount: Object.keys(validated as Record<string, unknown>).length
            });
            onFinish?.(validated);
          } catch (validationError) {
            // Keep the partial object but report validation error
            const err =
              validationError instanceof Error
                ? validationError
                : new Error(String(validationError));
            error.set(err);
            onError?.(err);
            logger.error('submit:validation-error', err, {
              requestId,
              api,
              durationMs: Date.now() - startedAt,
              partialObjectKeyCount: Object.keys(finalObject as Record<string, unknown>).length
            });
          }
        } else {
          logger.debug('submit:success-empty', {
            requestId,
            api,
            durationMs: Date.now() - startedAt
          });
        }
      } catch (err) {
        // Handle abort
        if (err instanceof Error && err.name === 'AbortError') {
          loading.set(false);
          logger.debug('submit:aborted', {
            requestId,
            api,
            durationMs: Date.now() - startedAt
          });
          return;
        }

        const errorInstance = err instanceof Error ? err : new Error(String(err));
        error.set(errorInstance);
        loading.set(false);
        onError?.(errorInstance);
        logger.error('submit:error', errorInstance, {
          requestId,
          api,
          durationMs: Date.now() - startedAt
        });
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
      logger.debug('stop', {
        api,
        isLoading: loading()
      });
      abortController.abort();
      abortController = null;
      loading.set(false);
    } else {
      logger.debug('stop:no-op', {
        api,
        reason: 'no active controller'
      });
    }
  };

  return {
    object,
    error,
    isLoading,
    submit,
    stop
  };
}
