import { fx } from "@flickjs/runtime";

// Reactive fx for current pathname
export const currentPath = fx(window.location.pathname);

// Reactive fx for route parameters
export const params = fx<Record<string, string>>({});

// Reactive fx for query string
export const queryParams = fx<URLSearchParams>(
  new URLSearchParams(window.location.search)
);
