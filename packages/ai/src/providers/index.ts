import type { LanguageModel } from 'ai';

import {
  getProviders as getProvidersFromRegistry,
  registerProvider as registerProviderInRegistry,
  resolveModel as resolveModelFromRegistry
} from '../server/providers/registry';

// Raw provider re-exports for power users
// Import from "@flickjs/ai/providers" when you need direct provider access
export { openai, createOpenAI } from '@ai-sdk/openai';
export { anthropic, createAnthropic } from '@ai-sdk/anthropic';
export { google, createGoogleGenerativeAI } from '@ai-sdk/google';
export { groq, createGroq } from '@ai-sdk/groq';
export { cerebras, createCerebras } from '@ai-sdk/cerebras';
export { createOpenRouter } from '@openrouter/ai-sdk-provider';

// Use a flexible type since providers may return different model versions
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ProviderFactory = (modelId: string) => any;

interface ParsedProviderSpec {
  provider?: string;
  modelId?: string;
}

function parseProviderSpec(spec: string): ParsedProviderSpec {
  const colonIndex = spec.indexOf(':');
  if (colonIndex === -1) {
    logProviderWarn('unrecognized provider spec format', {
      spec
    });
    return {};
  }

  return {
    provider: spec.slice(0, colonIndex),
    modelId: spec.slice(colonIndex + 1)
  };
}

function logProviderInfo(event: string, details: Record<string, unknown>): void {
  console.info(`[providers] ${event}`, details);
}

function logProviderError(event: string, details: Record<string, unknown>, error: unknown): void {
  console.error(`[providers] ${event}`, details, error);
}

function logProviderWarn(event: string, details: Record<string, unknown>): void {
  console.warn(`[providers] ${event}`, details);
}

function logProviderDebug(event: string, details: Record<string, unknown>): void {
  console.debug(`[providers] ${event}`, details);
}

// Registry utilities
export function resolveModel(spec: string | LanguageModel): LanguageModel {
  if (typeof spec !== 'string') {
    logProviderInfo('resolve passthrough model instance', {
      specType: typeof spec
    });
    return resolveModelFromRegistry(spec);
  }

  const { provider, modelId } = parseProviderSpec(spec);
  logProviderInfo('resolve requested', {
    spec,
    provider,
    modelId
  });

  try {
    const model = resolveModelFromRegistry(spec);
    logProviderInfo('provider selected', {
      spec,
      provider,
      modelId
    });
    return model;
  } catch (error) {
    logProviderError(
      'resolve failed',
      {
        spec,
        provider,
        modelId,
        message: error instanceof Error ? error.message : 'Unknown error'
      },
      error
    );
    throw error;
  }
}

export function registerProvider(name: string, factory: ProviderFactory): void {
  const existingProviders = getProvidersFromRegistry();
  const replaced = existingProviders.includes(name);

  logProviderInfo(replaced ? 're-registering provider' : 'registering provider', {
    provider: name
  });

  registerProviderInRegistry(name, factory);

  logProviderInfo('provider registered', {
    provider: name,
    replaced,
    totalProviders: getProvidersFromRegistry().length
  });
}

export function getProviders(): string[] {
  const providers = getProvidersFromRegistry();

  logProviderInfo('provider lookup', {
    count: providers.length
  });

  logProviderDebug('available provider names', {
    providers
  });

  return providers;
}
