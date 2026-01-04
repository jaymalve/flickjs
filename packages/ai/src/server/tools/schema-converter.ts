import { z } from "zod";
import type { SimpleSchema, SimpleSchemaType } from "./types";

/**
 * Convert a simple schema type string/object to a Zod type
 */
function convertType(type: SimpleSchemaType): z.ZodTypeAny {
  // Handle nested objects
  if (typeof type === "object" && !Array.isArray(type)) {
    return convertSchema(type as SimpleSchema);
  }

  // Handle arrays of objects
  if (Array.isArray(type)) {
    return z.array(convertSchema(type[0] as SimpleSchema));
  }

  // Handle optional suffix
  const isOptional = type.endsWith("?");
  const baseType = isOptional ? type.slice(0, -1) : type;

  let zodType: z.ZodTypeAny;

  switch (baseType) {
    case "string":
      zodType = z.string();
      break;
    case "number":
      zodType = z.number();
      break;
    case "boolean":
      zodType = z.boolean();
      break;
    case "string[]":
      zodType = z.array(z.string());
      break;
    case "number[]":
      zodType = z.array(z.number());
      break;
    case "boolean[]":
      zodType = z.array(z.boolean());
      break;
    default:
      zodType = z.unknown();
  }

  return isOptional ? zodType.optional() : zodType;
}

/**
 * Convert a simple schema object to a Zod object schema
 *
 * @example
 * ```ts
 * const zodSchema = convertSchema({
 *   name: "string",
 *   age: "number?",
 *   tags: "string[]",
 *   address: {
 *     city: "string",
 *     zip: "string?"
 *   }
 * });
 * ```
 */
export function convertSchema<T extends SimpleSchema>(
  schema: T
): z.ZodObject<Record<string, z.ZodTypeAny>> {
  const shape: Record<string, z.ZodTypeAny> = {};

  for (const [key, type] of Object.entries(schema)) {
    shape[key] = convertType(type as SimpleSchemaType);
  }

  return z.object(shape);
}

/**
 * Check if a value is a Zod schema
 */
export function isZodSchema(value: unknown): value is z.ZodType {
  return (
    value !== null &&
    typeof value === "object" &&
    "_def" in value &&
    typeof (value as z.ZodType)._def === "object"
  );
}
