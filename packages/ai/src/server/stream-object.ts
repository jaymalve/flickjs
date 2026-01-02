import { streamObject } from "ai";
import type { ObjectStreamOptions } from "./types";

/**
 * Create a streaming object response using the Vercel AI SDK
 *
 * @example
 * ```ts
 * // api/recipe.ts
 * import { createObjectStream } from "@flickjs/ai/server";
 * import { openai } from "@ai-sdk/openai";
 * import { z } from "zod";
 *
 * const recipeSchema = z.object({
 *   name: z.string(),
 *   ingredients: z.array(z.string()),
 *   steps: z.array(z.string()),
 * });
 *
 * export async function POST(req: Request) {
 *   const { input } = await req.json();
 *
 *   return createObjectStream({
 *     model: openai('gpt-4'),
 *     schema: recipeSchema,
 *     prompt: input,
 *   });
 * }
 * ```
 */
export async function createObjectStream<T>(
  options: ObjectStreamOptions<T>
): Promise<Response> {
  const {
    model,
    schema,
    system,
    prompt,
    messages,
    mode,
    maxTokens,
    temperature,
    abortSignal,
    headers,
    onFinish,
  } = options;

  const result = streamObject({
    model,
    schema,
    system,
    prompt,
    messages,
    mode,
    maxTokens,
    temperature,
    abortSignal,
    onFinish: onFinish
      ? (result: { object: unknown; finishReason: string; usage?: { promptTokens: number; completionTokens: number; totalTokens: number } }) => {
          onFinish({
            object: result.object as T,
            finishReason: result.finishReason as "stop" | "length" | "content-filter" | "tool-calls" | "error" | "other",
            usage: result.usage
              ? {
                  promptTokens: result.usage.promptTokens,
                  completionTokens: result.usage.completionTokens,
                  totalTokens: result.usage.totalTokens,
                }
              : undefined,
          });
        }
      : undefined,
  });

  return result.toTextStreamResponse({
    headers: headers ? Object.fromEntries(new Headers(headers)) : undefined,
  });
}
