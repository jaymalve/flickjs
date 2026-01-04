import { tool as aiTool } from "ai";
import type { z } from "zod";
import { convertSchema, isZodSchema } from "./schema-converter";
import type { SimpleSchema, SimpleToolOptions, ZodToolOptions } from "./types";

/**
 * Define a tool with simplified schema syntax
 *
 * Supports two modes:
 * 1. Simple schema (recommended) - Use plain object notation
 * 2. Zod schema (advanced) - Use full Zod schemas for complex validation
 *
 * @example Simple schema
 * ```ts
 * const weatherTool = tool({
 *   description: "Get current weather for a location",
 *   parameters: {
 *     location: "string",
 *     unit: "string?"  // optional
 *   },
 *   execute: async ({ location, unit }) => {
 *     return { temperature: 72, unit: unit || "fahrenheit" };
 *   }
 * });
 * ```
 *
 * @example Nested objects
 * ```ts
 * const createUserTool = tool({
 *   description: "Create a new user",
 *   parameters: {
 *     name: "string",
 *     age: "number?",
 *     address: {
 *       city: "string",
 *       country: "string"
 *     }
 *   },
 *   execute: async (user) => {
 *     return { id: "123", ...user };
 *   }
 * });
 * ```
 *
 * @example With Zod schema (advanced)
 * ```ts
 * import { z } from "zod";
 *
 * const weatherTool = tool({
 *   description: "Get current weather",
 *   schema: z.object({
 *     location: z.string(),
 *     unit: z.enum(["celsius", "fahrenheit"]).optional()
 *   }),
 *   execute: async ({ location, unit }) => {
 *     return { temperature: 72, unit: unit || "fahrenheit" };
 *   }
 * });
 * ```
 */
export function tool<TParams extends SimpleSchema, TResult>(
  options: SimpleToolOptions<TParams, TResult>
): ReturnType<typeof aiTool>;

export function tool<TSchema extends z.ZodType, TResult>(
  options: ZodToolOptions<TSchema, TResult>
): ReturnType<typeof aiTool>;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function tool(options: any): any {
  const { description, execute } = options;

  // Determine the schema - either provided Zod schema or convert simple schema
  let zodSchema: z.ZodType;

  if ("schema" in options && isZodSchema(options.schema)) {
    // Zod schema provided directly
    zodSchema = options.schema;
  } else if ("parameters" in options) {
    // Simple schema - convert to Zod
    zodSchema = convertSchema(options.parameters as SimpleSchema);
  } else {
    throw new Error("tool() requires either 'parameters' or 'schema'");
  }

  // Create the AI SDK tool
  return aiTool({
    description,
    parameters: zodSchema as z.ZodObject<Record<string, z.ZodTypeAny>>,
    execute: execute as (params: unknown) => Promise<unknown>,
  });
}
