// Client-side exports (default)
export { aiChat, aiObject } from "./client";
export type {
  Message,
  ChatStatus,
  AiChatOptions,
  AiChat,
  AiObjectOptions,
  AiObject,
} from "./client";

// Re-export stream parser utilities for advanced usage
export {
  parseStream,
  parseStreamLine,
  parseStreamWithCallbacks,
  extractTextFromStream,
} from "./utils/stream-parser";
export type { StreamPart, StreamPartType, StreamParserCallbacks } from "./utils/stream-parser";
