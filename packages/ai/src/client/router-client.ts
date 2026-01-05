import type { Agent, AgentResult } from "../server/agent/types";
import type { AgentRouter, InferAgentRouter } from "../router/types";
import type {
  AgentClient,
  AgentClientMethods,
  AgentClientOptions,
  ChatOptions,
  RunOptions,
  StreamOptions,
} from "./router-client-types";
import type { AiChat, Message } from "./types";
import { aiChat } from "./chat";

/**
 * Get the base URL for the agent API
 */
function getBaseUrl(options: AgentClientOptions): string {
  if (options.baseUrl) {
    return options.baseUrl;
  }

  // Check environment variables
  // Node.js / Bun
  if (typeof process !== "undefined" && process.env) {
    if (process.env.FLICK_AI_URL) {
      return process.env.FLICK_AI_URL;
    }
  }

  // Vite / browser with import.meta.env
  // We use a try-catch because import.meta.env may not exist in all environments
  try {
    // @ts-expect-error - import.meta.env is Vite-specific
    if (typeof import.meta !== "undefined" && import.meta.env?.VITE_FLICK_AI_URL) {
      // @ts-expect-error - import.meta.env is Vite-specific
      return import.meta.env.VITE_FLICK_AI_URL;
    }
  } catch {
    // import.meta not available
  }

  // Default fallback
  return "/api/ai";
}

/**
 * Create a proxy for a single agent that provides chat/run/stream methods
 */
function createAgentProxy(baseUrl: string, agentName: string): AgentClientMethods {
  const agentUrl = `${baseUrl}/${agentName}`;

  return {
    /**
     * Start a reactive chat session with the agent
     */
    chat(options: ChatOptions = {}): AiChat {
      return aiChat({
        api: agentUrl,
        ...options,
      });
    },

    /**
     * Run the agent without streaming - returns the complete result
     */
    async run(options: RunOptions): Promise<AgentResult> {
      const response = await fetch(agentUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify({
          messages: options.messages.map((m: Message) => ({
            role: m.role,
            content: m.content,
          })),
          stream: false,
        }),
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(error.error || `HTTP ${response.status}`);
      }

      return response.json();
    },

    /**
     * Start a streaming session with initial messages
     */
    stream(options: StreamOptions = {}): AiChat {
      return aiChat({
        api: agentUrl,
        initialMessages: options.messages,
      });
    },
  };
}

/**
 * Create a typed client for accessing agents defined in an AgentRouter
 *
 * @example
 * ```ts
 * // On the server (server/agents.ts)
 * import { agentRouter, agent } from "@flickjs/ai";
 *
 * export const agents = agentRouter({
 *   assistant: agent({ model: "openai:gpt-4o", system: "You are helpful." }),
 *   coder: agent({ model: "anthropic:claude-3-5-sonnet", system: "You help with code." })
 * });
 *
 * export type Agents = typeof agents;
 *
 * // On the client (client/ai.ts)
 * import { createAgentClient } from "@flickjs/ai";
 * import type { Agents } from "../server/agents";
 *
 * export const ai = createAgentClient<Agents>();
 *
 * // Usage - fully typed!
 * const chat = ai.assistant.chat();      // Returns reactive AiChat
 * const result = await ai.coder.run({    // Returns AgentResult
 *   messages: [{ role: "user", content: "Write a hello world" }]
 * });
 * ```
 *
 * @example With explicit baseUrl
 * ```ts
 * const ai = createAgentClient<Agents>({
 *   baseUrl: "https://api.example.com/ai"
 * });
 * ```
 *
 * @example Using environment variables
 * ```ts
 * // Set FLICK_AI_URL or VITE_FLICK_AI_URL in your environment
 * // The client will automatically use it
 * const ai = createAgentClient<Agents>();
 * ```
 */
export function createAgentClient<T extends AgentRouter<Record<string, Agent>>>(
  options: AgentClientOptions = {}
): AgentClient<InferAgentRouter<T>> {
  const baseUrl = getBaseUrl(options);

  // Use a Proxy to dynamically create agent accessors
  return new Proxy({} as AgentClient<InferAgentRouter<T>>, {
    get(_, agentName: string) {
      // Skip internal properties
      if (typeof agentName !== "string" || agentName.startsWith("_")) {
        return undefined;
      }
      return createAgentProxy(baseUrl, agentName);
    },

    // Support for Object.keys() and similar
    ownKeys() {
      return [];
    },

    getOwnPropertyDescriptor() {
      return {
        enumerable: true,
        configurable: true,
      };
    },
  });
}
