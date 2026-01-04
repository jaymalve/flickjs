import type { Agent } from "../server/agent/types";

/**
 * Agent router - a collection of named agents
 */
export interface AgentRouter<T extends Record<string, Agent>> {
  /** Internal map of agent names to agent instances */
  _agents: T;
  /** Type discriminator */
  _type: "agentRouter";
}

/**
 * Infer the agents record type from an AgentRouter
 */
export type InferAgentRouter<R> = R extends AgentRouter<infer T> ? T : never;
