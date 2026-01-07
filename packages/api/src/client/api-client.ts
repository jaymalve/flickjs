import { fx } from "@flickjs/runtime";
import type { ApiRouter } from "../router/types";
import type { Endpoint } from "../server/types";
import type {
  QueryResponse,
  MutationResponse,
  ApiClientOptions,
} from "./types";
import type { z } from "zod";

/**
 * Get the base URL for the API
 */
// need to find a better way to get the base url? - is hardcoding env key name okay?
function getBaseUrl(options: ApiClientOptions): string {
  if (options.baseUrl) {
    return options.baseUrl;
  }

  // Check environment variables
  if (typeof process !== "undefined" && process.env) {
    if (process.env.FLICK_API_URL) {
      return process.env.FLICK_API_URL;
    }
  }

  // Vite / browser with import.meta.env
  try {
    if (
      typeof import.meta !== "undefined" &&
      // @ts-expect-error - import.meta.env is Vite-specific
      import.meta.env?.VITE_FLICK_API_URL
    ) {
      // @ts-expect-error - import.meta.env is Vite-specific
      return import.meta.env.VITE_FLICK_API_URL;
    }
  } catch {
    // import.meta not available
  }

  // Default fallback
  return "/api";
}

/**
 * Create a reactive API response
 */
function createApiResponse<T>(
  execute: (input: any) => Promise<T>,
  input: any,
  isQuery: boolean
): QueryResponse<T> | MutationResponse<T> {
  const data = fx<T | null>(null);
  const loading = fx(false);
  const error = fx<string | null>(null);

  let lastInput: any = input;
  let abortController: AbortController | null = null;

  const runRequest = async (newInput?: any) => {
    if (newInput !== undefined) {
      lastInput = newInput;
    } else if (lastInput === null) {
      // If no input provided and we don't have one stored, use the initial input
      lastInput = input;
    }

    // Cancel previous request if still running
    if (abortController) {
      abortController.abort();
    }

    abortController = new AbortController();
    const currentController = abortController;

    loading.set(true);
    error.set(null);

    try {
      const result = await execute(lastInput);

      // Only update if this request wasn't cancelled
      if (!currentController.signal.aborted) {
        data.set(result);
        error.set(null);
        loading.set(false);
      }
    } catch (err) {
      // Only update if this request wasn't cancelled
      if (!currentController.signal.aborted) {
        const message = err instanceof Error ? err.message : "Unknown error";
        error.set(message);
        data.set(null);
        loading.set(false);
      }
    } finally {
      if (currentController === abortController) {
        abortController = null;
      }
    }
  };

  const retry = () => runRequest();

  const response: QueryResponse<T> | MutationResponse<T> = {
    data,
    loading,
    error,
    retry,
  } as any;

  if (isQuery) {
    (response as QueryResponse<T>).refetch = () => runRequest();
    // Execute immediately for queries
    runRequest();
  } else {
    // Mutations don't auto-execute - user must call retry() to trigger
    // Store the input for when retry() is called
    lastInput = input;
  }

  return response;
}

/**
 * Create a typed client for accessing endpoints defined in an ApiRouter
 *
 * @example
 * ```ts
 * // On the server (server/api.ts)
 * import { router, endpoint } from "@flickjs/api/server";
 *
 * export const api = router({
 *   users: {
 *     get: endpoint({ input: z.object({ id: z.string() }), query: ... }),
 *   }
 * });
 *
 * export type Api = typeof api;
 *
 * // On the client (client/api.ts)
 * import { createApiClient } from "@flickjs/api/client";
 * import type { Api } from "../server/api";
 * import { api as serverApi } from "../server/api";
 *
 * export const api = createApiClient<Api>({
 *   router: serverApi, // Pass router instance to access endpoint types
 * });
 *
 * // Usage - fully typed!
 * const user = api.users.get({ id: "123" });
 * // user.data() - Fx<User | null>
 * // user.loading() - Fx<boolean>
 * // user.error() - Fx<string | null>
 * ```
 */
export function createApiClient<T extends ApiRouter<any>>(
  options: ApiClientOptions = {}
): ApiClient<T> {
  const baseUrl = getBaseUrl(options);

  // Use a Proxy to dynamically create endpoint accessors
  return new Proxy({} as ApiClient<T>, {
    get(_, prop: string) {
      // Skip internal properties
      if (typeof prop !== "string" || prop.startsWith("_")) {
        return undefined;
      }

      // Create nested proxy for this path segment
      return createNestedProxy(baseUrl, prop, options);
    },

    ownKeys() {
      return [];
    },

    getOwnPropertyDescriptor() {
      return {
        enumerable: true,
        configurable: true, // not necessary to include this property
      };
    },
  });
}

/**
 * Get endpoint from router by path
 */
function getEndpointFromPath(
  router: ApiRouter<any> | undefined,
  path: string
): { _type: "query" | "mutation" } | null {
  if (!router || !router._endpoints) {
    return null;
  }

  const parts = path.split(".");
  let current: any = router._endpoints;

  for (const part of parts) {
    if (!current || typeof current !== "object") {
      return null;
    }

    // Access the property
    const next = current[part];

    if (!next || typeof next !== "object") {
      return null;
    }

    // If it's a nested router, access its _endpoints for the next iteration
    if ("_type" in next && next._type === "apiRouter" && "_endpoints" in next) {
      current = next._endpoints;
    } else {
      current = next;
    }
  }

  // Check if we found an endpoint
  if (
    current &&
    typeof current === "object" &&
    "_type" in current &&
    (current._type === "query" || current._type === "mutation")
  ) {
    return current as { _type: "query" | "mutation" };
  }

  return null;
}

/**
 * Create a nested proxy for router paths
 */
function createNestedProxy(
  baseUrl: string,
  currentPath: string,
  options: ApiClientOptions
): any {
  return new Proxy(
    {},
    {
      get(_, prop: string) {
        if (typeof prop !== "string" || prop.startsWith("_")) {
          return undefined;
        }

        const newPath = `${currentPath}.${prop}`;
        const url = `${baseUrl}/${newPath.replace(/\./g, "/")}`;

        // Return a function that creates the reactive response
        return (input: any = {}) => {
          // Get endpoint type from router instance
          const endpoint = getEndpointFromPath(options.router, newPath);
          const isQuery = endpoint?._type === "query";

          if (!endpoint) {
            throw new Error(
              `Cannot determine endpoint type for "${newPath}". ` +
                `Please provide the router instance to createApiClient: ` +
                `createApiClient<Api>({ router: serverApi })`
            );
          }

          const execute = async (reqInput: any) => {
            const headers: Record<string, string> = {
              "Content-Type": "application/json",
            };

            if (options.getHeaders) {
              const customHeaders = options.getHeaders();
              if (customHeaders instanceof Headers) {
                customHeaders.forEach((value, key) => {
                  headers[key] = value;
                });
              } else {
                Object.assign(headers, customHeaders);
              }
            }

            let response: Response;
            if (isQuery) {
              // GET request with query params
              const params = new URLSearchParams(
                Object.entries(reqInput || {}).reduce((acc, [key, value]) => {
                  acc[key] = String(value);
                  return acc;
                }, {} as Record<string, string>)
              ).toString();
              const fullUrl = params ? `${url}?${params}` : url;
              response = await fetch(fullUrl, {
                method: "GET",
                headers,
              });
            } else {
              // POST request with JSON body
              response = await fetch(url, {
                method: "POST",
                headers,
                body: JSON.stringify(reqInput || {}),
              });
            }

            if (!response.ok) {
              const errorData = await response.json().catch(() => ({
                error: response.statusText,
              }));
              throw new Error(errorData.error || `HTTP ${response.status}`);
            }

            const result = await response.json();
            return result.data;
          };

          return createApiResponse(execute, input, isQuery);
        };
      },
    }
  );
}

/**
 * Type helper for API client - maps router structure to client methods
 */
export type ApiClient<T> = T extends ApiRouter<infer E>
  ? {
      [K in keyof E]: E[K] extends Endpoint<infer I, infer O>
        ? I extends z.ZodTypeAny
          ? (
              input: z.infer<I>
            ) => E[K]["_type"] extends "query"
              ? QueryResponse<O>
              : MutationResponse<O>
          : never
        : E[K] extends ApiRouter<any>
        ? ApiClient<E[K]>
        : never;
    }
  : never;
