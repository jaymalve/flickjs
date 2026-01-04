import type { CoreMessage } from "ai";
import type { Agent } from "../server/agent/types";
import type { AgentRouter } from "./types";

/**
 * Create a fetch-compatible handler from an agent router
 *
 * Returns a function that handles HTTP requests and routes them to the appropriate agent.
 * Works with any runtime that supports the Web Fetch API (Bun, Deno, Cloudflare Workers, Vercel Edge, etc.)
 *
 * @example Bun
 * ```ts
 * import { createHandler, agentRouter, agent } from "@flickjs/ai";
 *
 * const agents = agentRouter({
 *   assistant: agent({ model: "openai:gpt-4o", system: "You are helpful." })
 * });
 *
 * Bun.serve({ port: 3000, fetch: createHandler(agents) });
 * ```
 *
 * @example Deno
 * ```ts
 * Deno.serve({ port: 3000 }, createHandler(agents));
 * ```
 *
 * @example Cloudflare Workers
 * ```ts
 * export default { fetch: createHandler(agents) };
 * ```
 *
 * @example Vercel Edge
 * ```ts
 * const handler = createHandler(agents);
 * export const GET = handler;
 * export const POST = handler;
 * ```
 *
 * @example Node.js + Hono
 * ```ts
 * import { Hono } from "hono";
 * import { serve } from "@hono/node-server";
 *
 * const app = new Hono();
 * const handler = createHandler(agents);
 * app.all("/ai/*", (c) => handler(c.req.raw));
 * serve({ fetch: app.fetch, port: 3000 });
 * ```
 */
export function createHandler<T extends Record<string, Agent>>(
  router: AgentRouter<T>
): (req: Request) => Promise<Response> {
  return async (req: Request): Promise<Response> => {
    // Extract agent name from URL path
    const url = new URL(req.url);
    const pathParts = url.pathname.split("/").filter(Boolean);
    const agentName = pathParts[pathParts.length - 1];

    // Find the agent
    const agent = router._agents[agentName];

    if (!agent) {
      return new Response(
        JSON.stringify({
          error: "Agent not found",
          agent: agentName,
          available: Object.keys(router._agents),
        }),
        {
          status: 404,
          headers: { "Content-Type": "application/json" },
        }
      );
    }

    // Parse request body
    let body: { messages?: CoreMessage[]; stream?: boolean };
    try {
      body = await req.json();
    } catch {
      return new Response(JSON.stringify({ error: "Invalid JSON body" }), {
        status: 400,
        headers: { "Content-Type": "application/json" },
      });
    }

    const { messages } = body;

    if (!messages || !Array.isArray(messages)) {
      return new Response(
        JSON.stringify({
          error: "Missing or invalid 'messages' array in request body",
        }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" },
        }
      );
    }

    // Cast to CoreMessage[] after validation
    const coreMessages = messages as CoreMessage[];

    // Determine if streaming or not
    // Stream by default, unless explicitly disabled or Accept header doesn't want it
    const acceptHeader = req.headers.get("accept") ?? "";
    const wantsStream =
      body.stream !== false &&
      (acceptHeader.includes("text/event-stream") ||
        acceptHeader.includes("*/*") ||
        acceptHeader === "");

    try {
      if (wantsStream) {
        // Streaming response
        return agent.chat(coreMessages);
      } else {
        // Non-streaming response
        const result = await agent.run(coreMessages);
        return new Response(JSON.stringify(result), {
          headers: { "Content-Type": "application/json" },
        });
      }
    } catch (error) {
      // Add this line for debugging
      console.error("[createHandler] Error:", error);
      const message = error instanceof Error ? error.message : "Unknown error";
      return new Response(JSON.stringify({ error: message }), {
        status: 500,
        headers: { "Content-Type": "application/json" },
      });
    }
  };
}
