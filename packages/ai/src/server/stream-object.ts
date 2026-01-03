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
    maxOutputTokens,
    temperature,
    abortSignal,
    headers,
    onFinish,
  } = options;

  const result = (streamObject as any)({
    model,
    schema,
    system,
    prompt,
    messages,
    mode,
    maxTokens,
    maxOutputTokens,
    temperature,
    abortSignal,
    onFinish: onFinish
      ? (event: any) => {
          // temporary fix for the finishReason type
          let finishReason:
            | "stop"
            | "length"
            | "content-filter"
            | "tool-calls"
            | "error"
            | "other" = "stop";

          if (event.error) {
            finishReason = "error";
          } else if (event.object === undefined) {
            finishReason = "other";
          }
          onFinish({
            object: event.object as T,
            finishReason: finishReason,
            usage: event.usage
              ? {
                  promptTokens: (event.usage as any).promptTokens ?? (event.usage as any).inputTokens ?? 0,
                  completionTokens: (event.usage as any).completionTokens ?? (event.usage as any).outputTokens ?? 0,
                  totalTokens: (event.usage as any).totalTokens ?? 0,
                }
              : undefined,
          });
        }
      : undefined,
  });

  return (await result).toDataStreamResponse({
    headers: headers ? Object.fromEntries(new Headers(headers)) : undefined,
  });
}
