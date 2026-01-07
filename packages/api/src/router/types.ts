import type { Endpoint } from "../server/types";

/**
 * API router - a collection of named endpoints
 */
export interface ApiRouter<T extends Record<string, Endpoint<any, any> | Record<string, any>>> {
  /** Internal map of endpoint names to endpoint instances */
  _endpoints: T;
  /** Type discriminator */
  _type: "apiRouter";
}

/**
 * Infer the endpoints record type from an ApiRouter
 */
export type InferRouter<R> = R extends ApiRouter<infer T> ? T : never;

/**
 * Extract endpoint type from router path
 */
export type ExtractEndpoint<
  TRouter,
  TPath extends string
> = TPath extends `${infer TFirst}.${infer TRest}`
  ? TFirst extends keyof TRouter
    ? TRouter[TFirst] extends ApiRouter<any>
      ? ExtractEndpoint<TRouter[TFirst]["_endpoints"], TRest>
      : never
    : never
  : TPath extends keyof TRouter
  ? TRouter[TPath] extends Endpoint<any, any>
    ? TRouter[TPath]
    : never
  : never;

