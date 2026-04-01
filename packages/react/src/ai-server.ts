// Server-side exports — NO 'use client' directive

// Agent and tool definitions
export { agent } from "@flickjs/ai";
export { tool } from "@flickjs/ai";
export type {
  Agent,
  AgentConfig,
  AgentChatOptions,
  AgentResult,
  ModelSpec,
} from "@flickjs/ai";
export type { ToolOptions } from "@flickjs/ai";

// Streaming response creators
export { createTextStream, createObjectStream } from "@flickjs/ai";
export type {
  TextStreamOptions,
  TextStreamResult,
  ObjectStreamOptions,
  ObjectStreamResult,
} from "@flickjs/ai";

// Router
export { agentRouter, createHandler, createExpressHandler } from "@flickjs/ai";
export type {
  AgentRouter,
  InferAgentRouter,
  HandlerOptions,
  CorsOptions,
} from "@flickjs/ai";

// Model resolution
export { resolveModel } from "@flickjs/ai";
