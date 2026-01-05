// ============================================
// Client-side exports
// ============================================
export { aiChat, aiObject } from "./client";
export type {
  Message,
  ChatStatus,
  AiChatOptions,
  AiChat,
  AiObjectOptions,
  AiObject,
} from "./client";

// Client router
export { createAgentClient } from "./client/router-client";
export type {
  AgentClient,
  AgentClientOptions,
  AgentClientMethods,
  RunOptions,
  StreamOptions,
  ChatOptions,
} from "./client/router-client-types";

// ============================================
// Server-side exports
// ============================================
export { agent } from "./server/agent";
export { tool } from "./server/tools";
export { createTextStream } from "./server/stream-text";
export { createObjectStream } from "./server/stream-object";
export { resolveModel } from "./server/providers";
export type {
  Agent,
  AgentConfig,
  AgentChatOptions,
  AgentResult,
  ModelSpec,
} from "./server/agent";
export type { ToolOptions } from "./server/tools";
export type {
  TextStreamOptions,
  TextStreamResult,
  ObjectStreamOptions,
  ObjectStreamResult,
} from "./server/types";

// ============================================
// Router exports
// ============================================
export { agentRouter, createHandler, createExpressHandler } from "./router";
export type { AgentRouter, InferAgentRouter, HandlerOptions, CorsOptions } from "./router";

// ============================================
// Utilities
// ============================================
// Re-export stream parser utilities for advanced usage
export {
  parseStream,
  parseStreamLine,
  parseStreamWithCallbacks,
  extractTextFromStream,
} from "./utils/stream-parser";
export type { StreamPart, StreamPartType, StreamParserCallbacks } from "./utils/stream-parser";

// Convenience re-exports from AI SDK
export type { CoreMessage, LanguageModel } from "ai";
