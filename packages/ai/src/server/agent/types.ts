import type { CoreMessage, LanguageModel } from "ai";

/**
 * Model specification - either a string shortcut or LanguageModel instance
 *
 * String formats:
 * - "provider:model" (e.g., "openai:gpt-4o-mini")
 * - Alias (e.g., "gpt-4o" -> "openai:gpt-4o")
 */
export type ModelSpec = string | LanguageModel;

/**
 * Agent configuration
 */
export interface AgentConfig {
  /** Model to use - string shortcut ("openai:gpt-4o-mini") or model instance */
  model: ModelSpec;

  /** System prompt */
  system?: string;

  /** Tools available to the agent */
  tools?: Record<string, unknown>;

  /** Maximum tool execution steps (default: 1) */
  maxSteps?: number;

  /** Default temperature for generation */
  temperature?: number;

  /** Maximum tokens to generate */
  maxTokens?: number;

  /** Tool choice configuration */
  toolChoice?: "auto" | "none" | "required" | { type: "tool"; toolName: string };
}

/**
 * Options for individual chat/run requests
 */
export interface AgentChatOptions {
  /** Override system prompt for this request */
  system?: string;

  /** Override temperature for this request */
  temperature?: number;

  /** Override max tokens for this request */
  maxTokens?: number;

  /** Abort signal for cancellation */
  abortSignal?: AbortSignal;

  /** Custom response headers */
  headers?: HeadersInit;

  /** Callback when generation finishes */
  onFinish?: (result: AgentResult) => void;

  /** Callback on error */
  onError?: (error: Error) => void;

  /** Callback on each text chunk */
  onText?: (text: string) => void;
}

/**
 * Result from agent execution
 */
export interface AgentResult {
  /** Generated text */
  text: string;

  /** Reason why generation stopped */
  finishReason:
    | "stop"
    | "length"
    | "tool-calls"
    | "content-filter"
    | "error"
    | "other";

  /** Token usage statistics */
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };

  /** Tool calls made during generation */
  toolCalls?: Array<{
    toolName: string;
    args: unknown;
    result: unknown;
  }>;
}

/**
 * Agent instance with chat/stream/run methods
 */
export interface Agent {
  /**
   * Stream a chat response
   * Returns a Response suitable for HTTP streaming
   */
  chat(messages: CoreMessage[], options?: AgentChatOptions): Promise<Response>;

  /**
   * Stream a chat response (alias for chat)
   */
  stream(
    messages: CoreMessage[],
    options?: AgentChatOptions
  ): Promise<Response>;

  /**
   * Non-streaming execution - returns final result
   */
  run(messages: CoreMessage[], options?: AgentChatOptions): Promise<AgentResult>;

  /**
   * Get the resolved model instance
   */
  getModel(): LanguageModel;

  /**
   * Get the agent configuration
   */
  getConfig(): AgentConfig;
}
