import type { z } from "zod";

/**
 * Simple schema type notation for tool parameters
 *
 * Supported types:
 * - "string", "number", "boolean" for primitives
 * - "string?", "number?", "boolean?" for optional primitives
 * - "string[]", "number[]", "boolean[]" for arrays
 * - "string[]?", "number[]?", "boolean[]?" for optional arrays
 * - Nested objects: { nested: { field: "string" } }
 * - Arrays of objects: [{ field: "string" }]
 */
export type SimpleSchemaType =
  | "string"
  | "string?"
  | "number"
  | "number?"
  | "boolean"
  | "boolean?"
  | "string[]"
  | "string[]?"
  | "number[]"
  | "number[]?"
  | "boolean[]"
  | "boolean[]?"
  | SimpleSchema
  | [SimpleSchema];

/**
 * Simple schema object - maps field names to their types
 */
export interface SimpleSchema {
  [key: string]: SimpleSchemaType;
}

/**
 * Infer TypeScript type from a simple schema type string
 */
type InferSimpleType<T extends SimpleSchemaType> = T extends "string"
  ? string
  : T extends "string?"
    ? string | undefined
    : T extends "number"
      ? number
      : T extends "number?"
        ? number | undefined
        : T extends "boolean"
          ? boolean
          : T extends "boolean?"
            ? boolean | undefined
            : T extends "string[]"
              ? string[]
              : T extends "string[]?"
                ? string[] | undefined
                : T extends "number[]"
                  ? number[]
                  : T extends "number[]?"
                    ? number[] | undefined
                    : T extends "boolean[]"
                      ? boolean[]
                      : T extends "boolean[]?"
                        ? boolean[] | undefined
                        : T extends SimpleSchema
                          ? InferSimpleSchema<T>
                          : T extends [SimpleSchema]
                            ? InferSimpleSchema<T[0]>[]
                            : unknown;

/**
 * Infer TypeScript type from a simple schema object
 */
export type InferSimpleSchema<T extends SimpleSchema> = {
  [K in keyof T as T[K] extends `${string}?` ? never : K]: InferSimpleType<
    T[K]
  >;
} & {
  [K in keyof T as T[K] extends `${string}?` ? K : never]?: InferSimpleType<
    T[K]
  >;
};

/**
 * Tool definition using simple schema syntax
 */
export interface SimpleToolOptions<
  TParams extends SimpleSchema,
  TResult = unknown,
> {
  /** Description of what the tool does */
  description: string;
  /** Parameters using simple schema notation */
  parameters: TParams;
  /** Function to execute when the tool is called */
  execute: (params: InferSimpleSchema<TParams>) => Promise<TResult> | TResult;
}

/**
 * Tool definition using Zod schema (advanced)
 */
export interface ZodToolOptions<TSchema extends z.ZodType, TResult = unknown> {
  /** Description of what the tool does */
  description: string;
  /** Zod schema for parameters */
  schema: TSchema;
  /** Function to execute when the tool is called */
  execute: (params: z.infer<TSchema>) => Promise<TResult> | TResult;
}

/**
 * Union type for tool options - supports both simple and Zod schemas
 */
export type ToolOptions<TParams, TResult = unknown> = TParams extends z.ZodType
  ? ZodToolOptions<TParams, TResult>
  : TParams extends SimpleSchema
    ? SimpleToolOptions<TParams, TResult>
    : never;
