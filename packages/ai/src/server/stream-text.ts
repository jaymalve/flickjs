import { streamText } from "ai";
import type { TextStreamOptions } from "./types";

/**
 * Create a streaming text response using the Vercel AI SDK
 *
 * @example
 * ```ts
 * // api/chat.ts
 * import { createTextStream } from "@flickjs/ai/server";
 * import { openai } from "@ai-sdk/openai";
 *
 * export async function POST(req: Request) {
 *   const { messages } = await req.json();
 *
 *   return createTextStream({
 *     model: openai('gpt-4'),
 *     system: 'You are a helpful assistant.',
 *     messages,
 *   });
 * }
 * ```
 */
export async function createTextStream(
  options: TextStreamOptions
): Promise<Response> {
  const {
    model,
    system,
    messages,
    maxTokens,
    temperature,
    topP,
    topK,
    presencePenalty,
    frequencyPenalty,
    stopSequences,
    abortSignal,
    headers,
    onStart,
    onText,
    onFinish,
  } = options;

  const result = streamText({
    model,
    system,
    messages,
    maxTokens,
    temperature,
    topP,
    topK,
    presencePenalty,
    frequencyPenalty,
    stopSequences,
    abortSignal,
    onChunk: onText
      ? ({ chunk }: { chunk: { type: string; textDelta?: string } }) => {
          if (chunk.type === "text-delta" && chunk.textDelta) {
            onText(chunk.textDelta);
          }
        }
      : undefined,
    onFinish: onFinish
      ? (result: {
          text: string;
          finishReason: string;
          usage?: {
            promptTokens: number;
            completionTokens: number;
            totalTokens: number;
          };
        }) => {
          onFinish({
            text: result.text,
            finishReason: result.finishReason as
              | "stop"
              | "length"
              | "content-filter"
              | "tool-calls"
              | "error"
              | "other",
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

  onStart?.();

  return (await result).toDataStreamResponse({
    headers: headers ? Object.fromEntries(new Headers(headers)) : undefined,
  });
}
