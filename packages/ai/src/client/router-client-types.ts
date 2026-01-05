import type { Agent, AgentResult } from "../server/agent/types";
import type { AiChat, AiChatOptions, Message } from "./types";

/**
 * Options for creating an agent client
 */
export interface AgentClientOptions {
  /**
   * Base URL for the agent API
   * Defaults to FLICK_AI_URL or VITE_FLICK_AI_URL environment variable, or "/api/ai"
   */
  baseUrl?: string;
}

/**
 * Options for running an agent (non-streaming)
 */
export interface RunOptions {
  /** Messages to send to the agent */
  messages: Message[];
}

/**
 * Options for streaming from an agent
 */
export interface StreamOptions {
  /** Initial messages for the stream */
  messages?: Message[];
}

/**
 * Options for chat (reactive streaming)
 */
export interface ChatOptions extends Omit<AiChatOptions, "api"> {
  // Inherits all AiChatOptions except api (which is set automatically)
}

/**
 * Methods available on each agent in the client
 */
export interface AgentClientMethods {
  /**
   * Start a reactive chat session with the agent
   * Returns an AiChat instance with reactive state
   */
  chat(options?: ChatOptions): AiChat;

  /**
   * Run the agent without streaming - returns the complete result
   */
  run(options: RunOptions): Promise<AgentResult>;

  /**
   * Start a streaming session with initial messages
   * Returns an AiChat instance
   */
  stream(options?: StreamOptions): AiChat;
}

/**
 * Typed client for accessing agents
 * Keys are the agent names, values are the client methods
 */
export type AgentClient<T extends Record<string, Agent>> = {
  [K in keyof T]: AgentClientMethods;
};
