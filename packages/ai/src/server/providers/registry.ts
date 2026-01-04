import type { LanguageModel } from "ai";
import { openai } from "@ai-sdk/openai";
import { anthropic } from "@ai-sdk/anthropic";
import { google } from "@ai-sdk/google";
import { groq } from "@ai-sdk/groq";
import { cerebras } from "@ai-sdk/cerebras";
import { createOpenRouter } from "@openrouter/ai-sdk-provider";

// Use a flexible type since providers may return different model versions
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ProviderFactory = (modelId: string) => any;

const openrouter = createOpenRouter();

const providers: Record<string, ProviderFactory> = {
  openai: (modelId) => openai(modelId),
  anthropic: (modelId) => anthropic(modelId),
  google: (modelId) => google(modelId),
  groq: (modelId) => groq(modelId),
  cerebras: (modelId) => cerebras(modelId),
  openrouter: (modelId) => openrouter(modelId),
};

// Common model aliases for convenience
const modelAliases: Record<string, string> = {
  // OpenAI
  "gpt-4o": "openai:gpt-4o",
  "gpt-4o-mini": "openai:gpt-4o-mini",
  "gpt-4-turbo": "openai:gpt-4-turbo",
  "gpt-4": "openai:gpt-4",
  "o1": "openai:o1",
  "o1-mini": "openai:o1-mini",
  "o1-preview": "openai:o1-preview",
  // Anthropic
  "claude-3-opus": "anthropic:claude-3-opus-20240229",
  "claude-3-sonnet": "anthropic:claude-3-sonnet-20240229",
  "claude-3-haiku": "anthropic:claude-3-haiku-20240307",
  "claude-3.5-sonnet": "anthropic:claude-3-5-sonnet-20241022",
  "claude-3.5-haiku": "anthropic:claude-3-5-haiku-20241022",
  "claude-sonnet-4": "anthropic:claude-sonnet-4-20250514",
  "claude-opus-4": "anthropic:claude-opus-4-20250514",
  // Google
  "gemini-2.0-flash": "google:gemini-2.0-flash",
  "gemini-1.5-pro": "google:gemini-1.5-pro",
  "gemini-1.5-flash": "google:gemini-1.5-flash",
  // Groq
  "llama-3.3-70b": "groq:llama-3.3-70b-versatile",
  "llama-3.1-8b": "groq:llama-3.1-8b-instant",
  "mixtral-8x7b": "groq:mixtral-8x7b-32768",
  // Cerebras
  "llama-3.3-70b-cerebras": "cerebras:llama-3.3-70b",
  "llama-3.1-8b-cerebras": "cerebras:llama-3.1-8b",
};

/**
 * Resolve a model string to a LanguageModel instance
 *
 * @example
 * resolveModel("openai:gpt-4o-mini")  // OpenAI model
 * resolveModel("anthropic:claude-3-5-sonnet-20241022")  // Anthropic model
 * resolveModel("gpt-4o")  // Alias lookup -> openai:gpt-4o
 */
export function resolveModel(spec: string | LanguageModel): LanguageModel {
  // If already a model instance, return as-is
  if (typeof spec !== "string") {
    return spec;
  }

  // Check for alias first
  const resolved = modelAliases[spec] || spec;

  // Parse provider:model format
  const colonIndex = resolved.indexOf(":");
  if (colonIndex === -1) {
    throw new Error(
      `Invalid model spec "${spec}". Expected format: "provider:model" (e.g., "openai:gpt-4o-mini") or use an alias like "gpt-4o"`
    );
  }

  const provider = resolved.slice(0, colonIndex);
  const modelId = resolved.slice(colonIndex + 1);

  const factory = providers[provider];
  if (!factory) {
    throw new Error(
      `Unknown provider "${provider}". Available providers: ${Object.keys(providers).join(", ")}`
    );
  }

  return factory(modelId);
}

/**
 * Register a custom provider
 */
export function registerProvider(name: string, factory: ProviderFactory): void {
  providers[name] = factory;
}

/**
 * Add a model alias
 */
export function registerAlias(alias: string, spec: string): void {
  modelAliases[alias] = spec;
}

/**
 * Get list of available providers
 */
export function getProviders(): string[] {
  return Object.keys(providers);
}

/**
 * Get list of available aliases
 */
export function getAliases(): Record<string, string> {
  return { ...modelAliases };
}
