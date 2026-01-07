import type { z } from "zod";

/**
 * Endpoint configuration
 */
export interface EndpointConfig<TInput extends z.ZodTypeAny, TOutput> {
  /** Zod schema for input validation */
  input: TInput;
  /** Query handler (GET) - read-only operation */
  query?: (input: z.infer<TInput>, ctx: any) => Promise<TOutput> | TOutput;
  /** Mutation handler (POST) - write operation */
  mutation?: (input: z.infer<TInput>, ctx: any) => Promise<TOutput> | TOutput;
}

/**
 * Endpoint instance
 */
export interface Endpoint<TInput extends z.ZodTypeAny, TOutput> {
  /** Input schema */
  _input: TInput;
  /** Output type */
  _output: TOutput;
  /** Whether it's a query or mutation */
  _type: "query" | "mutation";
  /** Handler function */
  _handler: (input: z.infer<TInput>, ctx: any) => Promise<TOutput> | TOutput;
}

/**
 * Context factory function
 */
export type ContextFactory<TContext = any> = (req: Request) => Promise<TContext> | TContext;

