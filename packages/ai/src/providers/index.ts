// Raw provider re-exports for power users
// Import from "@flickjs/ai/providers" when you need direct provider access
export { openai, createOpenAI } from "@ai-sdk/openai";
export { anthropic, createAnthropic } from "@ai-sdk/anthropic";
export { google, createGoogleGenerativeAI } from "@ai-sdk/google";
export { groq, createGroq } from "@ai-sdk/groq";
export { cerebras, createCerebras } from "@ai-sdk/cerebras";
export { createOpenRouter } from "@openrouter/ai-sdk-provider";

// Registry utilities
export {
  resolveModel,
  registerProvider,
  getProviders,
} from "../server/providers/registry";
