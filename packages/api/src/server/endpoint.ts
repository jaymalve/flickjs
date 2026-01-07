import type { z } from "zod";
import type { Endpoint, EndpointConfig } from "./types";

/**
 * Create an API endpoint with input validation and handler
 *
 * @example Query endpoint
 * ```ts
 * const getUser = endpoint({
 *   input: z.object({ id: z.string() }),
 *   query: async ({ id }, ctx) => {
 *     return db.users.find(id);
 *   }
 * });
 * ```
 *
 * @example Mutation endpoint
 * ```ts
 * const createUser = endpoint({
 *   input: z.object({ name: z.string() }),
 *   mutation: async ({ name }, ctx) => {
 *     return db.users.create({ name });
 *   }
 * });
 * ```
 */
export function endpoint<
  TInput extends z.ZodTypeAny,
  TOutput = unknown
>(config: EndpointConfig<TInput, TOutput>): Endpoint<TInput, TOutput> {
  const { input, query, mutation } = config;

  if (!query && !mutation) {
    throw new Error("Endpoint must have either 'query' or 'mutation' handler");
  }

  if (query && mutation) {
    throw new Error("Endpoint cannot have both 'query' and 'mutation' handlers");
  }

  const handler = query || mutation!;
  const type = query ? "query" : "mutation";

  return {
    _input: input,
    _output: undefined as any, // Type inference only
    _type: type,
    _handler: handler,
  };
}

