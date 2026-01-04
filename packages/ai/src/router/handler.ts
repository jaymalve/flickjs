import type { CoreMessage } from "ai";
import type { Agent } from "../server/agent/types";
import type { AgentRouter } from "./types";

/**
 * Options for createHandler
 */
export interface HandlerOptions {
  /**
   * Enable CORS headers for cross-origin requests
   * Useful when frontend and backend run on different ports during development
   * @default true
   */
  cors?: boolean | CorsOptions;
}

/**
 * CORS configuration options
 */
export interface CorsOptions {
  /** Allowed origins - defaults to "*" */
  origin?: string | string[];
  /** Allowed methods - defaults to ["GET", "POST", "OPTIONS"] */
  methods?: string[];
  /** Allowed headers - defaults to ["Content-Type"] */
  allowedHeaders?: string[];
}

/**
 * Get CORS headers based on options
 */
function getCorsHeaders(cors: boolean | CorsOptions): Record<string, string> {
  if (cors === false) return {};

  const options: CorsOptions = cors === true ? {} : cors;

  const origin = Array.isArray(options.origin)
    ? options.origin.join(", ")
    : options.origin ?? "*";

  const methods = options.methods?.join(", ") ?? "GET, POST, OPTIONS";
  const headers = options.allowedHeaders?.join(", ") ?? "Content-Type";

  return {
    "Access-Control-Allow-Origin": origin,
    "Access-Control-Allow-Methods": methods,
    "Access-Control-Allow-Headers": headers,
  };
}

/**
 * Add CORS headers to an existing Response
 */
function withCors(response: Response, corsHeaders: Record<string, string>): Response {
  if (Object.keys(corsHeaders).length === 0) return response;

  const newHeaders = new Headers(response.headers);
  for (const [key, value] of Object.entries(corsHeaders)) {
    newHeaders.set(key, value);
  }

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: newHeaders,
  });
}

/**
 * Create a fetch-compatible handler from an agent router
 *
 * Returns a function that handles HTTP requests and routes them to the appropriate agent.
 * Works with any runtime that supports the Web Fetch API (Bun, Deno, Cloudflare Workers, Vercel Edge, etc.)
 *
 * @example Bun with CORS (default)
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
 * @example Disable CORS in production
 * ```ts
 * Bun.serve({
 *   port: 3000,
 *   fetch: createHandler(agents, { cors: false })
 * });
 * ```
 *
 * @example Custom CORS options
 * ```ts
 * Bun.serve({
 *   port: 3000,
 *   fetch: createHandler(agents, {
 *     cors: { origin: "https://myapp.com" }
 *   })
 * });
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
 */
export function createHandler<T extends Record<string, Agent>>(
  router: AgentRouter<T>,
  options: HandlerOptions = {}
): (req: Request) => Promise<Response> {
  const { cors = true } = options;
  const corsHeaders = getCorsHeaders(cors);

  return async (req: Request): Promise<Response> => {
    // Handle CORS preflight requests
    if (req.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: corsHeaders,
      });
    }

    // Extract agent name from URL path
    const url = new URL(req.url);
    const pathParts = url.pathname.split("/").filter(Boolean);
    const agentName = pathParts[pathParts.length - 1];

    // Find the agent
    const agent = router._agents[agentName];

    if (!agent) {
      return withCors(
        new Response(
          JSON.stringify({
            error: "Agent not found",
            agent: agentName,
            available: Object.keys(router._agents),
          }),
          {
            status: 404,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
      );
    }

    // Parse request body
    let body: { messages?: CoreMessage[]; stream?: boolean };
    try {
      body = await req.json();
    } catch {
      return withCors(
        new Response(JSON.stringify({ error: "Invalid JSON body" }), {
          status: 400,
          headers: { "Content-Type": "application/json" },
        }),
        corsHeaders
      );
    }

    const { messages } = body;

    if (!messages || !Array.isArray(messages)) {
      return withCors(
        new Response(
          JSON.stringify({
            error: "Missing or invalid 'messages' array in request body",
          }),
          {
            status: 400,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
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
        const response = await agent.chat(coreMessages);
        return withCors(response, corsHeaders);
      } else {
        // Non-streaming response
        const result = await agent.run(coreMessages);
        return withCors(
          new Response(JSON.stringify(result), {
            headers: { "Content-Type": "application/json" },
          }),
          corsHeaders
        );
      }
    } catch (error) {
      console.error("[createHandler] Error:", error);
      const message = error instanceof Error ? error.message : "Unknown error";
      return withCors(
        new Response(JSON.stringify({ error: message }), {
          status: 500,
          headers: { "Content-Type": "application/json" },
        }),
        corsHeaders
      );
    }
  };
}
