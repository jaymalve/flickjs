import type { z } from "zod";

/**
 * Tool definition options using Zod schema
 */
export interface ToolOptions<TSchema extends z.ZodType, TResult = unknown> {
  /** Description of what the tool does */
  description: string;
  /** Zod schema for parameters */
  parameters: TSchema;
  /** Function to execute when the tool is called */
  execute: (params: z.infer<TSchema>) => Promise<TResult> | TResult;
}
