import type { Endpoint } from "./types";
import type { ApiRouter, InferRouter } from "../router/types";
import type { z } from "zod";

/**
 * Create an API router - a collection of named endpoints
 *
 * @example
 * ```ts
 * import { router, endpoint } from "@flickjs/api";
 * import { z } from "zod";
 *
 * const getUser = endpoint({
 *   input: z.object({ id: z.string() }),
 *   query: async ({ id }, ctx) => db.users.find(id)
 * });
 *
 * const createUser = endpoint({
 *   input: z.object({ name: z.string() }),
 *   mutation: async ({ name }, ctx) => db.users.create({ name })
 * });
 *
 * export const api = router({
 *   users: {
 *     get: getUser,
 *     create: createUser,
 *   }
 * });
 *
 * export type Api = typeof api;
 * ```
 */
export function router<
  T extends Record<string, Endpoint<any, any> | Record<string, any>>
>(endpoints: T): ApiRouter<T> {
  return {
    _endpoints: endpoints,
    _type: "apiRouter" as const,
  };
}

/**
 * Infer the router type
 */
export type InferApiRouter<T> = InferRouter<T>;
