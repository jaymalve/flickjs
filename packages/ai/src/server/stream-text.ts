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
    maxOutputTokens,
    temperature,
    topP,
    topK,
    presencePenalty,
    frequencyPenalty,
    stopSequences,
    abortSignal,
    headers,
    tools,
    toolChoice,
    maxSteps,
    onStart,
    onText,
    onFinish,
    onError,
  } = options;

  console.log("[createTextStream] Starting with options:", {
    hasModel: !!model,
    hasSystem: !!system,
    messagesCount: messages?.length,
    hasTools: !!tools,
    toolNames: tools ? Object.keys(tools) : [],
    toolChoice,
    maxSteps,
  });

  const result = (streamText as any)({
    model,
    system,
    messages,
    maxTokens,
    maxOutputTokens,
    temperature,
    topP,
    topK,
    presencePenalty,
    frequencyPenalty,
    stopSequences,
    abortSignal,
    tools,
    toolChoice,
    maxSteps,
    onStepFinish: (step: any) => {
      console.log("[createTextStream] Step finished:", {
        stepType: step.stepType,
        text: step.text?.substring(0, 100),
        toolCalls: step.toolCalls,
        toolResults: step.toolResults,
        finishReason: step.finishReason,
      });
    },
    onChunk: onText
      ? ({ chunk }: { chunk: { type: string; textDelta?: string } }) => {
          if (chunk.type === "text-delta" && chunk.textDelta) {
            onText(chunk.textDelta);
          }
        }
      : undefined,
    onFinish: onFinish
      ? (event: any) => {
          onFinish({
            text: event.text,
            finishReason: event.finishReason as
              | "stop"
              | "length"
              | "content-filter"
              | "tool-calls"
              | "error"
              | "other",
            usage: event.usage
              ? {
                  promptTokens:
                    (event.usage as any).promptTokens ??
                    (event.usage as any).inputTokens ??
                    0,
                  completionTokens:
                    (event.usage as any).completionTokens ??
                    (event.usage as any).outputTokens ??
                    0,
                  totalTokens: (event.usage as any).totalTokens ?? 0,
                }
              : undefined,
          });
        }
      : undefined,
  });

  onStart?.();

  try {
    const awaited = await result;
    console.log("[createTextStream] Stream awaited successfully");
    return awaited.toDataStreamResponse({
      headers: headers ? Object.fromEntries(new Headers(headers)) : undefined,
    });
  } catch (error) {
    console.error("[createTextStream] Error:", error);
    throw error;
  }
}
