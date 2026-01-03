import type { Fx } from "@flickjs/runtime";
import type { z } from "zod";

/**
 * Represents a chat message
 */
export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  createdAt?: Date;
}

/**
 * Status of the AI chat
 */
export type ChatStatus = "idle" | "submitting" | "streaming" | "error";

/**
 * Options for aiChat()
 */
export interface AiChatOptions {
  /** API endpoint to send messages to */
  api: string;

  /** Initial messages to populate the chat */
  initialMessages?: Message[];

  /** Initial input value */
  initialInput?: string;

  /** Headers to send with requests */
  headers?: HeadersInit;

  /** Request body to merge with messages */
  body?: Record<string, unknown>;

  /** Called when a message is finished streaming */
  onFinish?: (message: Message) => void;

  /** Called when an error occurs */
  onError?: (error: Error) => void;

  /** Called when a response is received */
  onResponse?: (response: Response) => void;

  /** Whether to integrate with Suspense boundaries */
  suspense?: boolean;

  /** Generate message ID */
  generateId?: () => string;

  /** Credentials mode for fetch */
  credentials?: RequestCredentials;
}

/**
 * Return type of aiChat()
 */
export interface AiChat {
  /** Reactive list of messages */
  messages: Fx<Message[]>;

  /** Reactive input value */
  input: Fx<string>;

  /** Reactive status */
  status: Fx<ChatStatus>;

  /** Reactive error state */
  error: Fx<Error | undefined>;

  /** Derived: whether a request is in progress */
  isLoading: () => boolean;

  /** Submit a message (uses input if not provided) */
  submit: (message?: string) => Promise<void>;

  /** Stop the current stream */
  stop: () => void;

  /** Reload the last assistant message */
  reload: () => Promise<void>;

  /** Set messages directly */
  setMessages: (messages: Message[] | ((prev: Message[]) => Message[])) => void;

  /** Form submit handler */
  handleSubmit: (event: Event) => void;

  /** Input change handler */
  handleInputChange: (event: Event) => void;
}

/**
 * Options for aiObject()
 */
export interface AiObjectOptions<T> {
  /** API endpoint */
  api: string;

  /** Zod schema for the object */
  schema: z.ZodType<T>;

  /** Headers to send with requests */
  headers?: HeadersInit;

  /** Request body to merge with input */
  body?: Record<string, unknown>;

  /** Called when streaming is complete */
  onFinish?: (object: T) => void;

  /** Called when an error occurs */
  onError?: (error: Error) => void;

  /** Whether to integrate with Suspense boundaries */
  suspense?: boolean;

  /** Credentials mode for fetch */
  credentials?: RequestCredentials;
}

/**
 * Return type of aiObject()
 */
export interface AiObject<T> {
  /** Reactive partial object as it streams in */
  object: Fx<Partial<T> | undefined>;

  /** Reactive error state */
  error: Fx<Error | undefined>;

  /** Derived: whether a request is in progress */
  isLoading: () => boolean;

  /** Submit input to generate the object */
  submit: (input: string | Record<string, unknown>) => Promise<void>;

  /** Stop the current stream */
  stop: () => void;
}
