import type { ApiRouter } from "./types";
import type { Endpoint } from "../server/types";
import type { ContextFactory } from "../server/types";

/**
 * Minimal Express types to avoid requiring express as a dependency
 */
interface ExpressRequest {
  method: string;
  url: string;
  headers: Record<string, string | string[] | undefined>;
  body?: unknown;
  protocol: string;
  get(name: string): string | undefined;
}

interface ExpressResponse {
  status(code: number): ExpressResponse;
  set(headers: Record<string, string>): ExpressResponse;
  json(body: unknown): void;
  send(body: string | Buffer): void;
  setHeader(name: string, value: string): void;
  write(chunk: unknown): boolean;
  end(): void;
  headersSent: boolean;
}

type ExpressNextFunction = (err?: unknown) => void;

/**
 * Options for createApiHandler
 */
export interface HandlerOptions {
  /**
   * Context factory function
   */
  createContext?: ContextFactory<any>;
  /**
   * Enable CORS headers for cross-origin requests
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
 * Find endpoint in router by path
 */
function findEndpoint(
  router: ApiRouter<any>,
  pathParts: string[]
): { endpoint: Endpoint<any, any>; path: string } | null {
  let current: any = router._endpoints;
  const fullPath: string[] = [];

  for (const part of pathParts) {
    if (!current || typeof current !== "object") {
      return null;
    }

    // Check if current level has _type === "apiRouter"
    if (current._type === "apiRouter") {
      current = current._endpoints;
    }

    if (!(part in current)) {
      return null;
    }

    fullPath.push(part);
    current = current[part];
  }

  // Check if we found an endpoint
  if (current && current._type && (current._type === "query" || current._type === "mutation")) {
    return {
      endpoint: current as Endpoint<any, any>,
      path: fullPath.join("."),
    };
  }

  return null;
}

/**
 * Create a fetch-compatible handler from an API router
 *
 * @example Bun
 * ```ts
 * import { createApiHandler, router } from "@flickjs/api";
 *
 * const api = router({ ... });
 * Bun.serve({ port: 3000, fetch: createApiHandler(api) });
 * ```
 *
 * @example With context
 * ```ts
 * const handler = createApiHandler(api, {
 *   createContext: async (req) => ({ user: await getUser(req) })
 * });
 * ```
 */
export function createApiHandler<T extends Record<string, any>>(
  router: ApiRouter<T>,
  options: HandlerOptions = {}
): (req: Request) => Promise<Response> {
  const { cors = true, createContext } = options;
  const corsHeaders = getCorsHeaders(cors);

  return async (req: Request): Promise<Response> => {
    // Handle CORS preflight requests
    if (req.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: corsHeaders,
      });
    }

    // Extract endpoint path from URL
    const url = new URL(req.url);
    const pathParts = url.pathname.split("/").filter(Boolean);

    // Remove base path if it's "/api" or similar
    if (pathParts[0] === "api") {
      pathParts.shift();
    }

    if (pathParts.length === 0) {
      return withCors(
        new Response(
          JSON.stringify({
            error: "No endpoint specified",
            available: Object.keys(router._endpoints),
          }),
          {
            status: 400,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
      );
    }

    // Find the endpoint
    const found = findEndpoint(router, pathParts);
    if (!found) {
      return withCors(
        new Response(
          JSON.stringify({
            error: "Endpoint not found",
            path: pathParts.join("/"),
            available: Object.keys(router._endpoints),
          }),
          {
            status: 404,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
      );
    }

    const { endpoint } = found;

    // Validate HTTP method
    const expectedMethod = endpoint._type === "query" ? "GET" : "POST";
    if (req.method !== expectedMethod) {
      return withCors(
        new Response(
          JSON.stringify({
            error: `Method ${req.method} not allowed. Expected ${expectedMethod}`,
          }),
          {
            status: 405,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
      );
    }

    // Parse and validate input
    let input: any = {};

    if (req.method === "GET") {
      // For GET requests, parse query params
      const params = Object.fromEntries(url.searchParams.entries());
      input = params;
    } else {
      // For POST requests, parse body
      try {
        input = await req.json();
      } catch {
        return withCors(
          new Response(JSON.stringify({ error: "Invalid JSON body" }), {
            status: 400,
            headers: { "Content-Type": "application/json" },
          }),
          corsHeaders
        );
      }
    }

    // Validate input with Zod schema
    let validatedInput: any;
    try {
      validatedInput = endpoint._input.parse(input);
    } catch (error: any) {
      return withCors(
        new Response(
          JSON.stringify({
            error: "Validation error",
            details: error.errors || error.message,
          }),
          {
            status: 400,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
      );
    }

    // Create context
    const ctx = createContext ? await createContext(req) : {};

    // Execute handler
    try {
      const result = await endpoint._handler(validatedInput, ctx);
      return withCors(
        new Response(JSON.stringify({ data: result }), {
          headers: { "Content-Type": "application/json" },
        }),
        corsHeaders
      );
    } catch (error) {
      console.error("[createApiHandler] Error:", error);
      const message = error instanceof Error ? error.message : "Unknown error";
      return withCors(
        new Response(
          JSON.stringify({
            error: message,
          }),
          {
            status: 500,
            headers: { "Content-Type": "application/json" },
          }
        ),
        corsHeaders
      );
    }
  };
}

/**
 * Create an Express-compatible middleware from an API router
 */
export function createExpressHandler<T extends Record<string, any>>(
  router: ApiRouter<T>,
  options: HandlerOptions = {}
): (req: ExpressRequest, res: ExpressResponse, next: ExpressNextFunction) => Promise<void> {
  const { cors = true, createContext } = options;
  const corsHeaders = getCorsHeaders(cors);
  const fetchHandler = createApiHandler(router, options);

  return async (req, res, _next) => {
    // Apply CORS headers
    for (const [key, value] of Object.entries(corsHeaders)) {
      res.setHeader(key, value);
    }

    // Handle CORS preflight
    if (req.method === "OPTIONS") {
      res.status(204).end();
      return;
    }

    // Convert Express request to Fetch Request
    const url = `${req.protocol}://${req.get("host")}${req.url}`;
    const headers = new Headers();
    Object.entries(req.headers).forEach(([key, value]) => {
      if (value) {
        headers.set(key, Array.isArray(value) ? value.join(", ") : value);
      }
    });

    const body = req.body ? JSON.stringify(req.body) : undefined;
    const fetchReq = new Request(url, {
      method: req.method,
      headers,
      body,
    });

    try {
      const response = await fetchHandler(fetchReq);
      res.status(response.status);

      // Copy headers
      response.headers.forEach((value, key) => {
        res.setHeader(key, value);
      });

      // Send body
      const text = await response.text();
      res.send(text);
    } catch (error) {
      if (!res.headersSent) {
        res.status(500).json({
          error: error instanceof Error ? error.message : "Unknown error",
        });
      }
    }
  };
}

