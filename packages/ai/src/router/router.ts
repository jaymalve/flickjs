import type { Agent } from '../server/agent/types';
import type { AgentRouter } from './types';

type RouterLogLevel = 'info' | 'warn' | 'error' | 'debug';

let routerRequestId = 0;

export function nextRouterRequestId(): number {
  routerRequestId += 1;
  logAgentRouterEvent('request ID generated', {
    requestId: routerRequestId
  }, 'debug');
  return routerRequestId;
}

export function describeAgentModel(agent: Agent): string {
  const model = agent.getConfig().model;
  return typeof model === 'string' ? model : '[LanguageModel instance]';
}

export function logAgentRouterEvent(
  event: string,
  details: Record<string, unknown>,
  level: RouterLogLevel = 'info',
  error?: unknown
): void {
  const prefix = `[agentRouter] ${event}`;

  if (level === 'warn') {
    if (error !== undefined) {
      console.warn(prefix, details, error);
      return;
    }

    console.warn(prefix, details);
    return;
  }

  if (level === 'error') {
    if (error !== undefined) {
      console.error(prefix, details, error);
      return;
    }

    console.error(prefix, details);
    return;
  }

  if (level === 'debug') {
    console.debug(prefix, details);
    return;
  }

  console.info(prefix, details);
}

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
export function agentRouter<T extends Record<string, Agent>>(agents: T): AgentRouter<T> {
  const agentNames = Object.keys(agents);

  if (agentNames.length === 0) {
    logAgentRouterEvent('router created with no agents', {}, 'warn');
  }

  const agentModels: Record<string, string> = {};
  for (const name of agentNames) {
    agentModels[name] = describeAgentModel(agents[name]);
  }

  logAgentRouterEvent('router created', {
    agentCount: agentNames.length,
    agents: agentModels
  });

  return {
    _agents: agents,
    _type: 'agentRouter' as const
  };
}
