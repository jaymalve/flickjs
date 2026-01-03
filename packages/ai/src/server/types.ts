import type { LanguageModel, CoreMessage } from "ai";

/**
 * Options for createTextStream()
 */
export interface TextStreamOptions {
  /** The language model to use */
  model: LanguageModel;

  /** System prompt */
  system?: string;

  /** Messages to send to the model */
  messages: CoreMessage[];

  /** Maximum tokens to generate (AI SDK v4) */
  maxTokens?: number;

  /** Maximum tokens to generate (AI SDK v5) */
  maxOutputTokens?: number;

  /** Temperature for generation */
  temperature?: number;

  /** Top-p sampling */
  topP?: number;

  /** Top-k sampling */
  topK?: number;

  /** Presence penalty */
  presencePenalty?: number;

  /** Frequency penalty */
  frequencyPenalty?: number;

  /** Stop sequences */
  stopSequences?: string[];

  /** Abort signal */
  abortSignal?: AbortSignal;

  /** Additional headers for the response */
  headers?: HeadersInit;

  /** Tools available to the model - use tool() from 'ai' package to define */
  tools?: Record<string, unknown>;

  /** How the model should choose tools: 'auto', 'none', 'required', or { type: 'tool', toolName: string } */
  toolChoice?:
    | "auto"
    | "none"
    | "required"
    | { type: "tool"; toolName: string };

  /** Maximum number of steps for multi-step tool execution */
  maxSteps?: number;

  /** Called when generation starts */
  onStart?: () => void;

  /** Called for each text chunk */
  onText?: (text: string) => void;

  /** Called when generation finishes */
  onFinish?: (result: TextStreamResult) => void;

  /** Called when an error occurs */
  onError?: (error: Error) => void;
}

/**
 * Result passed to onFinish callback
 */
export interface TextStreamResult {
  /** The full generated text */
  text: string;

  /** Reason why generation stopped */
  finishReason:
    | "stop"
    | "length"
    | "content-filter"
    | "tool-calls"
    | "error"
    | "other";

  /** Token usage statistics */
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
}

/**
 * Options for createObjectStream()
 */
export interface ObjectStreamOptions<T> {
  /** The language model to use */
  model: LanguageModel;

  /** Zod schema for the object */
  schema: import("zod").ZodType<T>;

  /** System prompt */
  system?: string;

  /** The prompt or messages */
  prompt?: string;
  messages?: CoreMessage[];

  /** Schema mode - 'auto' lets the model decide */
  mode?: "auto" | "json" | "tool";

  /** Maximum tokens to generate (AI SDK v4) */
  maxTokens?: number;

  /** Maximum tokens to generate (AI SDK v5) */
  maxOutputTokens?: number;

  /** Temperature for generation */
  temperature?: number;

  /** Abort signal */
  abortSignal?: AbortSignal;

  /** Additional headers for the response */
  headers?: HeadersInit;

  /** Called when generation finishes */
  onFinish?: (result: ObjectStreamResult<T>) => void;
}

/**
 * Result passed to onFinish callback for object streams
 */
export interface ObjectStreamResult<T> {
  /** The generated object */
  object: T;

  /** Reason why generation stopped */
  finishReason:
    | "stop"
    | "length"
    | "content-filter"
    | "tool-calls"
    | "error"
    | "other";

  /** Token usage statistics */
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
}
