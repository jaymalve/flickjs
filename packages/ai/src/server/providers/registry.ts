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

/**
 * Resolve a model string to a LanguageModel instance
 *
 * @example
 * resolveModel("openai:gpt-4o-mini")
 * resolveModel("anthropic:claude-3-5-sonnet-20241022")
 * resolveModel("openrouter:anthropic/claude-3.5-sonnet")
 * resolveModel("groq:llama-3.3-70b-versatile")
 */
export function resolveModel(spec: string | LanguageModel): LanguageModel {
  // If already a model instance, return as-is
  if (typeof spec !== "string") {
    return spec;
  }

  // Parse provider:model format
  const colonIndex = spec.indexOf(":");
  if (colonIndex === -1) {
    throw new Error(
      `Invalid model spec "${spec}". Expected format: "provider:model" (e.g., "openai:gpt-4o-mini")`
    );
  }

  const provider = spec.slice(0, colonIndex);
  const modelId = spec.slice(colonIndex + 1);

  const factory = providers[provider];
  if (!factory) {
    throw new Error(
      `Unknown provider "${provider}". Available: ${Object.keys(providers).join(", ")}`
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
 * Get list of available providers
 */
export function getProviders(): string[] {
  return Object.keys(providers);
}
