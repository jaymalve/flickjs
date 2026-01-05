import type { Agent } from "../server/agent/types";
import type { AgentRouter } from "./types";

/**
 * Create an agent router - a collection of named agents
 *
 * @example
 * ```ts
 * import { agentRouter, agent } from "@flickjs/ai";
 *
 * export const agents = agentRouter({
 *   assistant: agent({
 *     model: "openai:gpt-4o",
 *     system: "You are a helpful assistant."
 *   }),
 *   coder: agent({
 *     model: "anthropic:claude-3-5-sonnet",
 *     system: "You are a coding assistant."
 *   })
 * });
 *
 * export type Agents = typeof agents;
 * ```
 */
export function agentRouter<T extends Record<string, Agent>>(
  agents: T
): AgentRouter<T> {
  return {
    _agents: agents,
    _type: "agentRouter" as const,
  };
}
