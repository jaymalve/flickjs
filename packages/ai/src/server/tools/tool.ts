import { tool as aiTool } from "ai";
import type { z } from "zod";
import type { ToolOptions } from "./types";

/**
 * Define a tool with Zod schema for parameters
 *
 * @example
 * ```ts
 * import { tool } from "@flickjs/ai/server";
 * import { z } from "zod";
 *
 * const weatherTool = tool({
 *   description: "Get current weather for a location",
 *   parameters: z.object({
 *     location: z.string(),
 *     unit: z.enum(["celsius", "fahrenheit"]).optional()
 *   }),
 *   execute: async ({ location, unit }) => {
 *     return { temperature: 72, unit: unit || "fahrenheit" };
 *   }
 * });
 * ```
 */
export function tool<TSchema extends z.ZodType, TResult>(
  options: ToolOptions<TSchema, TResult>
) {
  const { description, parameters, execute } = options;

  return aiTool({
    description,
    parameters: parameters as unknown as z.ZodObject<
      Record<string, z.ZodTypeAny>
    >,
    execute: execute as (params: unknown) => Promise<unknown>,
  });
}
