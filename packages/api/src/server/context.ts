import type { ContextFactory } from "./types";

/**
 * Create a context factory that extracts context from request
 *
 * @example Basic context
 * ```ts
 * export const createContext = createContextFactory(async (req) => {
 *   return {
 *     user: await getUserFromRequest(req),
 *   };
 * });
 * ```
 */
export function createContextFactory<TContext>(
  factory: ContextFactory<TContext>
): ContextFactory<TContext> {
  return factory;
}

