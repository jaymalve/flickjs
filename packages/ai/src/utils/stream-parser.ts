/**
 * Stream parser for Vercel AI SDK Data Stream Protocol
 *
 * Supports both legacy prefix format (0:, e:, d:) and
 * the newer SSE-based protocol with JSON objects.
 */

export type StreamPartType =
  | "text"
  | "data"
  | "error"
  | "tool_call"
  | "tool_result"
  | "finish"
  | "message_id"
  | "unknown";

export interface StreamPart {
  type: StreamPartType;
  value: unknown;
}

/**
 * Legacy prefix codes from AI SDK Data Stream Protocol
 */
const LEGACY_PREFIX_MAP: Record<string, StreamPartType> = {
  "0": "text", // Text content
  "2": "data", // Custom data
  "3": "error", // Error
  "9": "tool_call", // Tool call
  "a": "tool_result", // Tool result (hex for 10)
  "e": "finish", // Finish event
  "d": "finish", // Done signal (alternative finish)
  "f": "message_id", // Message ID
};

/**
 * Parse a single line from the legacy prefix format
 * Format: PREFIX:JSON_VALUE (e.g., 0:"Hello" or e:{"finishReason":"stop"})
 */
function parseLegacyLine(line: string): StreamPart | null {
  if (line.length < 2) return null;

  const prefix = line[0];
  if (line[1] !== ":") return null;

  const type = LEGACY_PREFIX_MAP[prefix];
  if (!type) {
    return { type: "unknown", value: line.slice(2) };
  }

  const jsonPart = line.slice(2);
  try {
    const value = JSON.parse(jsonPart);
    return { type, value };
  } catch {
    // For text, the value might be a raw string without quotes
    return { type, value: jsonPart };
  }
}

/**
 * Parse a single line from the SSE format
 * Format: data: {"type":"text-delta","delta":"..."} or data: [DONE]
 */
function parseSSELine(line: string): StreamPart | null {
  if (!line.startsWith("data: ")) return null;

  const data = line.slice(6); // Remove "data: " prefix

  if (data === "[DONE]") {
    return { type: "finish", value: { finishReason: "stop" } };
  }

  try {
    const parsed = JSON.parse(data);

    // Map SSE types to our StreamPartType
    switch (parsed.type) {
      case "text-delta":
        return { type: "text", value: parsed.delta };
      case "text-start":
      case "text-end":
        return { type: "text", value: "" };
      case "error":
        return { type: "error", value: parsed.errorText || parsed.error };
      case "finish":
        return { type: "finish", value: parsed };
      case "tool-input-available":
        return { type: "tool_call", value: parsed };
      case "tool-output-available":
        return { type: "tool_result", value: parsed };
      case "start":
        return { type: "message_id", value: parsed.messageId };
      default:
        // Handle data-* types
        if (parsed.type?.startsWith("data-")) {
          return { type: "data", value: parsed.data };
        }
        return { type: "unknown", value: parsed };
    }
  } catch {
    return null;
  }
}

/**
 * Parse a line from the stream (auto-detects format)
 */
export function parseStreamLine(line: string): StreamPart | null {
  const trimmed = line.trim();
  if (!trimmed) return null;

  // Try SSE format first (starts with "data: ")
  if (trimmed.startsWith("data:")) {
    return parseSSELine(trimmed);
  }

  // Try legacy prefix format (single char + colon)
  if (trimmed.length >= 2 && trimmed[1] === ":") {
    return parseLegacyLine(trimmed);
  }

  return null;
}

/**
 * Create an async iterator that parses stream chunks
 */
export async function* parseStream(
  reader: ReadableStreamDefaultReader<Uint8Array>
): AsyncGenerator<StreamPart> {
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();

    if (done) {
      // Process any remaining buffer
      if (buffer.trim()) {
        const part = parseStreamLine(buffer);
        if (part) yield part;
      }
      break;
    }

    buffer += decoder.decode(value, { stream: true });

    // Process complete lines
    const lines = buffer.split("\n");
    buffer = lines.pop() || ""; // Keep incomplete line in buffer

    for (const line of lines) {
      const part = parseStreamLine(line);
      if (part) yield part;
    }
  }
}

/**
 * Callback-based stream parser for simpler usage
 */
export interface StreamParserCallbacks {
  onText?: (text: string) => void;
  onData?: (data: unknown) => void;
  onError?: (error: string) => void;
  onFinish?: (reason: string) => void;
  onToolCall?: (toolCall: unknown) => void;
  onToolResult?: (result: unknown) => void;
}

export async function parseStreamWithCallbacks(
  reader: ReadableStreamDefaultReader<Uint8Array>,
  callbacks: StreamParserCallbacks
): Promise<void> {
  for await (const part of parseStream(reader)) {
    switch (part.type) {
      case "text":
        callbacks.onText?.(String(part.value));
        break;
      case "data":
        callbacks.onData?.(part.value);
        break;
      case "error":
        callbacks.onError?.(String(part.value));
        break;
      case "finish":
        const finishData = part.value as { finishReason?: string } | undefined;
        callbacks.onFinish?.(finishData?.finishReason || "stop");
        break;
      case "tool_call":
        callbacks.onToolCall?.(part.value);
        break;
      case "tool_result":
        callbacks.onToolResult?.(part.value);
        break;
    }
  }
}

/**
 * Extract text content from a stream, accumulating into a string
 */
export async function extractTextFromStream(
  reader: ReadableStreamDefaultReader<Uint8Array>
): Promise<string> {
  let text = "";

  await parseStreamWithCallbacks(reader, {
    onText: (chunk) => {
      text += chunk;
    },
  });

  return text;
}
