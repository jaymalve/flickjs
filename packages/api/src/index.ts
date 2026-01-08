// ============================================
// Server-side exports
// ============================================
export { endpoint, router, createContextFactory } from "./server";
export type {
  Endpoint,
  EndpointConfig,
  ContextFactory,
  InferApiRouter,
} from "./server";

// ============================================
// Router exports
// ============================================
export { createApiHandler, createExpressHandler } from "./router";
export type {
  HandlerOptions,
  CorsOptions,
  ApiRouter,
  InferRouter,
} from "./router";

// ============================================
// Client-side exports
// ============================================
export { createApiClient } from "@flickjs/runtime/api-client";
export type {
  ApiClient,
  ApiResponse,
  QueryResponse,
  MutationResponse,
  ApiClientOptions,
} from "@flickjs/runtime/api-client";
