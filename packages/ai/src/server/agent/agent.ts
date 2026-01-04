import { streamText, generateText, type LanguageModel } from "ai";
import type { CoreMessage } from "ai";
import { resolveModel } from "../providers/registry";
import type { Agent, AgentConfig, AgentChatOptions, AgentResult } from "./types";

/**
 * Create a reusable AI agent with pre-configured settings
 *
 * @example Basic usage
 * ```ts
 * import { agent } from "@flickjs/ai/server";
 *
 * const assistant = agent({
 *   model: "openai:gpt-4o-mini",
 *   system: "You are a helpful assistant.",
 * });
 *
 * // In API route
 * export async function POST(req: Request) {
 *   const { messages } = await req.json();
 *   return assistant.chat(messages);
 * }
 * ```
 *
 * @example With tools
 * ```ts
 * import { agent, tool } from "@flickjs/ai/server";
 *
 * const weatherTool = tool({
 *   description: "Get weather for a location",
 *   parameters: { location: "string" },
 *   execute: async ({ location }) => ({ temp: 72, condition: "sunny" })
 * });
 *
 * const assistant = agent({
 *   model: "openai:gpt-4o-mini",
 *   system: "You are a weather assistant.",
 *   tools: { weather: weatherTool },
 *   maxSteps: 5
 * });
 *
 * return assistant.chat(messages);
 * ```
 *
 * @example Non-streaming
 * ```ts
 * const result = await assistant.run(messages);
 * console.log(result.text);
 * console.log(result.usage);
 * ```
 */
export function agent(config: AgentConfig): Agent {
  const {
    model: modelSpec,
    system,
    tools,
    maxSteps = 1,
    temperature,
    maxTokens,
    toolChoice,
  } = config;

  // Resolve model once at creation time
  const model = resolveModel(modelSpec);

  /**
   * Stream a chat response
   */
  async function chat(
    messages: CoreMessage[],
    options: AgentChatOptions = {}
  ): Promise<Response> {
    const result = streamText({
      model,
      system: options.system ?? system,
      messages,
      tools: tools as Parameters<typeof streamText>[0]["tools"],
      maxSteps,
      toolChoice,
      temperature: options.temperature ?? temperature,
      maxTokens: options.maxTokens ?? maxTokens,
      abortSignal: options.abortSignal,
      onChunk: options.onText
        ? ({ chunk }) => {
            if (chunk.type === "text-delta" && chunk.textDelta) {
              options.onText!(chunk.textDelta);
            }
          }
        : undefined,
      onFinish: options.onFinish
        ? (event) => {
            options.onFinish!({
              text: event.text,
              finishReason: event.finishReason as AgentResult["finishReason"],
              usage: event.usage
                ? {
                    promptTokens:
                      (event.usage as Record<string, number>).promptTokens ??
                      (event.usage as Record<string, number>).inputTokens ??
                      0,
                    completionTokens:
                      (event.usage as Record<string, number>).completionTokens ??
                      (event.usage as Record<string, number>).outputTokens ??
                      0,
                    totalTokens:
                      (event.usage as Record<string, number>).totalTokens ?? 0,
                  }
                : undefined,
            });
          }
        : undefined,
    });

    try {
      const awaited = await result;
      return awaited.toDataStreamResponse({
        headers: options.headers
          ? Object.fromEntries(new Headers(options.headers))
          : undefined,
      });
    } catch (error) {
      if (options.onError && error instanceof Error) {
        options.onError(error);
      }
      throw error;
    }
  }

  /**
   * Non-streaming execution
   */
  async function run(
    messages: CoreMessage[],
    options: AgentChatOptions = {}
  ): Promise<AgentResult> {
    try {
      const result = await generateText({
        model,
        system: options.system ?? system,
        messages,
        tools: tools as Parameters<typeof generateText>[0]["tools"],
        maxSteps,
        toolChoice,
        temperature: options.temperature ?? temperature,
        maxTokens: options.maxTokens ?? maxTokens,
        abortSignal: options.abortSignal,
      });

      return {
        text: result.text,
        finishReason: result.finishReason as AgentResult["finishReason"],
        usage: result.usage
          ? {
              promptTokens:
                (result.usage as Record<string, number>).promptTokens ??
                (result.usage as Record<string, number>).inputTokens ??
                0,
              completionTokens:
                (result.usage as Record<string, number>).completionTokens ??
                (result.usage as Record<string, number>).outputTokens ??
                0,
              totalTokens:
                (result.usage as Record<string, number>).totalTokens ?? 0,
            }
          : undefined,
      };
    } catch (error) {
      if (options.onError && error instanceof Error) {
        options.onError(error);
      }
      throw error;
    }
  }

  return {
    chat,
    stream: chat, // Alias
    run,
    getModel: () => model,
    getConfig: () => config,
  };
}
