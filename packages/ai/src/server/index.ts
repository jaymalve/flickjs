// Existing streaming APIs
export { createTextStream } from "./stream-text";
export { createObjectStream } from "./stream-object";
export type {
  TextStreamOptions,
  TextStreamResult,
  ObjectStreamOptions,
  ObjectStreamResult,
} from "./types";

// Provider re-exports
export {
  openai,
  createOpenAI,
  anthropic,
  createAnthropic,
  google,
  createGoogleGenerativeAI,
  groq,
  createGroq,
  cerebras,
  createCerebras,
  createOpenRouter,
  resolveModel,
  registerProvider,
  registerAlias,
  getProviders,
  getAliases,
} from "./providers";

// Tool helpers
export { tool, convertSchema, isZodSchema } from "./tools";
export type {
  SimpleSchema,
  SimpleSchemaType,
  SimpleToolOptions,
  ZodToolOptions,
  ToolOptions,
  InferSimpleSchema,
} from "./tools";

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
