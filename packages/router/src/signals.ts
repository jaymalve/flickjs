import { signal } from "@flickjs/runtime";

// Reactive signal for current pathname
export const currentPath = signal(window.location.pathname);

// Reactive signal for route parameters
export const params = signal<Record<string, string>>({});

// Reactive signal for query string
export const query = signal<URLSearchParams>(
  new URLSearchParams(window.location.search)
);
