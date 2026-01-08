# @flickjs/api

Type-safe API client/server for FlickJS with reactive responses.

## Installation

```bash
# For server-side usage (no runtime needed)
bun add @flickjs/api zod
# or
npm install @flickjs/api zod

# For client-side usage (includes runtime)
bun add @flickjs/runtime zod
# or
npm install @flickjs/runtime zod
```

## Features

- **End-to-end type safety** - Full TypeScript inference from server to client
- **Reactive responses** - All API calls return reactive objects with `{ data, loading, error, refetch, retry }`
- **Minimal API** - Simple `endpoint({ input, query/mutation })` pattern
- **Deploy anywhere** - Works with Bun, Deno, Cloudflare Workers, Vercel Edge, Express, Node.js
- **Zod validation** - Required Zod schemas for runtime + type safety

## Quick Start

### Server Setup

```typescript
// server/api.ts
import { endpoint, router } from "@flickjs/api";
import { z } from "zod";

const getUser = endpoint({
  input: z.object({ id: z.string() }),
  query: async ({ id }, ctx) => {
    return { id, name: "John", email: "john@example.com" };
  },
});

const createUser = endpoint({
  input: z.object({ name: z.string(), email: z.string() }),
  mutation: async ({ name, email }, ctx) => {
    return { id: "123", name, email };
  },
});

export const api = router({
  users: {
    get: getUser,
    create: createUser,
  },
});

export type Api = typeof api;
```

### Server Handler

```typescript
// server/index.ts
import { createApiHandler } from "@flickjs/api";
import { api } from "./api";

const handler = createApiHandler(api, {
  createContext: async (req) => {
    // Extract user from request, etc.
    return { user: null };
  },
});

// Deploy anywhere!
export default handler;

// Bun
// Bun.serve({ port: 3000, fetch: handler });

// Deno
// Deno.serve({ port: 3000 }, handler);

// Cloudflare Workers
// export default { fetch: handler };

// Express
// import express from "express";
// const app = express();
// app.use(express.json());
// app.use("/api", createExpressHandler(api));
// app.listen(3000);
```

### Client Setup

```typescript
// client/api.ts
import { createApiClient } from "@flickjs/api";
// Alternative: import { createApiClient } from "@flickjs/runtime/api-client";
import type { Api } from "../server/api";

export const api = createApiClient<Api>({
  baseUrl: "/api", // or process.env.API_URL
  getHeaders: () => {
    const token = localStorage.getItem("auth_token");
    return token ? { Authorization: `Bearer ${token}` } : {};
  },
});
```

### Client Usage

```typescript
import { api } from "./client/api";

// Query - returns reactive object
const user = api.users.get({ id: "123" });

// Access reactive values
if (user.loading()) return <div>Loading...</div>;
if (user.error()) return <div>Error: {user.error()}</div>;
return <div>{user.data()?.name}</div>;

// Manual refetch/retry
await user.refetch(); // Re-run query
await user.retry(); // Retry if failed

// Mutation
const createResult = api.users.create({
  name: "John",
  email: "john@example.com",
});

// Mutations don't auto-execute, call retry() to trigger
await createResult.retry();
```

## API Reference

### `endpoint(config)`

Create an API endpoint.

**Parameters:**

- `config.input` - Zod schema for input validation (required)
- `config.query` - Handler function for GET requests (read-only)
- `config.mutation` - Handler function for POST requests (write operations)

**Example:**

```typescript
const getUser = endpoint({
  input: z.object({ id: z.string() }),
  query: async ({ id }, ctx) => {
    return db.users.find(id);
  },
});
```

### `router(endpoints)`

Create an API router from a collection of endpoints.

**Example:**

```typescript
export const api = router({
  users: {
    get: getUser,
    create: createUser,
  },
});
```

### `createApiHandler(router, options)`

Create a fetch-compatible request handler.

**Options:**

- `createContext` - Function to create request context
- `cors` - CORS configuration (boolean or CorsOptions)

### `createApiClient<Api>(options)`

Create a typed API client.

**Options:**

- `baseUrl` - Base URL for API requests (default: "/api")
- `getHeaders` - Function to get headers for each request

**Returns:** Reactive API client with full type safety

## Reactive Responses

All API calls return a reactive object:

```typescript
interface ApiResponse<T> {
  data: Fx<T | null>; // Result data
  loading: Fx<boolean>; // Loading state
  error: Fx<string | null>; // Error message
  refetch?: () => Promise<void>; // Re-run query (queries only)
  retry: () => Promise<void>; // Retry failed request
}
```

**Queries** include `refetch()` and `retry()`  
**Mutations** include `retry()` only (no `refetch()`)

## Context

Create request context for authentication, database connections, etc.:

```typescript
const handler = createApiHandler(api, {
  createContext: async (req) => {
    const token = req.headers.get("authorization")?.replace("Bearer ", "");
    const user = token ? await verifyUser(token) : null;

    return {
      user, // Can be null if no token provided
      db: getDatabase(), // Should always return a valid DB connection
    };
  },
});
```

**Important:** `createContext` can return `null` values. These are passed directly to endpoints, so endpoints must handle null checks:

```typescript
const getUser = endpoint({
  input: z.object({ id: z.string() }),
  query: async ({ id }, ctx) => {
    // Must check for null - context doesn't reject early
    if (!ctx.user) {
      throw new Error("Unauthorized"); // Returns 500 error
    }
    if (!ctx.db) {
      throw new Error("Database unavailable");
    }
    return ctx.db.users.find(id);
  },
});
```

**Note:** Currently, `createContext` cannot reject requests early. If you need early rejection (e.g., return 401 instead of 500), you'll need to check in the endpoint handler or use middleware (coming soon).

## Type Safety

Full type inference from server to client:

```typescript
// Server
export const api = router({
  users: {
    get: endpoint({
      input: z.object({ id: z.string() }),
      query: async ({ id }) => ({ id, name: "John" }),
    }),
  },
});

export type Api = typeof api;

// Client - fully typed!
const api = createApiClient<Api>();
const user = api.users.get({ id: "123" });
// user.data() is typed as Fx<{ id: string; name: string } | null>
```

## Deployment

Works with all major JavaScript runtimes:

- **Bun**: `Bun.serve({ fetch: createApiHandler(api) })`
- **Deno**: `Deno.serve(createApiHandler(api))`
- **Cloudflare Workers**: `export default { fetch: createApiHandler(api) }`
- **Vercel Edge**: Export handler as route handler
- **Express**: `app.use("/api", createExpressHandler(api))`
- **Node.js**: With fetch polyfill or Express adapter

## License

MIT
