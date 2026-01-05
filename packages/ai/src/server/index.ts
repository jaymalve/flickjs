// Existing streaming APIs
export { createTextStream } from "./stream-text";
export { createObjectStream } from "./stream-object";
export type {
  TextStreamOptions,
  TextStreamResult,
  ObjectStreamOptions,
  ObjectStreamResult,
} from "./types";

// Model resolution (used internally by agent, also exported for convenience)
export { resolveModel } from "./providers";

// Tool helpers
export { tool } from "./tools";
export type { ToolOptions } from "./tools";

// Agent API
export { agent } from "./agent";
export type {
  Agent,
  AgentConfig,
  AgentChatOptions,
  AgentResult,
  ModelSpec,
} from "./agent";

// Convenience re-exports from AI SDK
export type { CoreMessage, LanguageModel } from "ai";
