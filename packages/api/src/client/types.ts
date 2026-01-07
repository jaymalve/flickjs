import type { Fx } from "@flickjs/runtime";
import type { ApiRouter } from "../router/types";

/**
 * Reactive API response
 */
export interface ApiResponse<T> {
  /** Result data (null initially) */
  data: Fx<T | null>;
  /** Is request in progress? */
  loading: Fx<boolean>;
  /** Error message if any (null if no error) */
  error: Fx<string | null>;
  /** Re-run query with same input (queries only) */
  refetch?: () => Promise<void>;
  /** Retry failed request (queries & mutations) */
  retry: () => Promise<void>;
}

/**
 * Query response (includes refetch)
 */
export interface QueryResponse<T> extends ApiResponse<T> {
  refetch: () => Promise<void>;
}

/**
 * Mutation response (no refetch)
 */
export interface MutationResponse<T> extends ApiResponse<T> {
  refetch?: never;
}

/**
 * Client options
 */
export interface ApiClientOptions {
  /** Base URL for API requests */
  baseUrl?: string;
  /** Function to get headers for each request */
  getHeaders?: () => Record<string, string> | HeadersInit;
  /** Router instance to access endpoint metadata */
  router?: ApiRouter<any>;
}
