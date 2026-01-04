// Re-export provider functions for convenience
export { openai, createOpenAI } from "@ai-sdk/openai";
export { anthropic, createAnthropic } from "@ai-sdk/anthropic";
export { google, createGoogleGenerativeAI } from "@ai-sdk/google";
export { groq, createGroq } from "@ai-sdk/groq";
export { cerebras, createCerebras } from "@ai-sdk/cerebras";
export { createOpenRouter } from "@openrouter/ai-sdk-provider";

// Re-export registry utilities
export {
  resolveModel,
  registerProvider,
  registerAlias,
  getProviders,
  getAliases,
} from "./registry";
