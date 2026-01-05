import { fx, getCurrentSuspense } from "@flickjs/runtime";
import type { AiChat, AiChatOptions, ChatStatus, Message } from "./types";
import { parseStream } from "../utils/stream-parser";

/**
 * Generate a unique ID for messages
 */
function generateId(): string {
  return Math.random().toString(36).substring(2, 9);
}

/**
 * Create a reactive AI chat instance
 *
 * @example
 * ```tsx
 * const chat = aiChat({ api: '/api/chat' });
 *
 * // Reactive state
 * chat.messages()     // Message[]
 * chat.input()        // string
 * chat.status()       // 'idle' | 'submitting' | 'streaming' | 'error'
 * chat.isLoading()    // boolean
 *
 * // Actions
 * chat.submit('Hello!')
 * chat.stop()
 * chat.reload()
 * ```
 */
export function aiChat(options: AiChatOptions): AiChat {
  const {
    api,
    initialMessages = [],
    initialInput = "",
    headers,
    body,
    onFinish,
    onError,
    onResponse,
    suspense = false,
    generateId: customGenerateId = generateId,
    credentials,
  } = options;

  // Reactive state
  const messages = fx<Message[]>(initialMessages);
  const input = fx<string>(initialInput);
  const status = fx<ChatStatus>("idle");
  const error = fx<Error | undefined>(undefined);

  // Track current abort controller for cancellation
  let abortController: AbortController | null = null;

  /**
   * Derived loading state
   */
  const isLoading = (): boolean => {
    const s = status();
    return s === "submitting" || s === "streaming";
  };

  /**
   * Submit a message to the chat
   */
  const submit = async (message?: string): Promise<void> => {
    const content = message ?? input();

    if (!content.trim()) return;

    // Clear input immediately
    if (!message) {
      input.set("");
    }

    // Create user message
    const userMessage: Message = {
      id: customGenerateId(),
      role: "user",
      content: content.trim(),
      createdAt: new Date(),
    };

    // Add user message to list
    messages.set((prev: Message[]) => [...prev, userMessage]);

    // Create placeholder for assistant response
    const assistantMessage: Message = {
      id: customGenerateId(),
      role: "assistant",
      content: "",
      createdAt: new Date(),
    };

    messages.set((prev: Message[]) => [...prev, assistantMessage]);

    // Update status
    status.set("submitting");
    error.set(undefined);

    // Create abort controller
    abortController = new AbortController();

    // Create promise for Suspense integration
    const streamPromise = (async () => {
      try {
        const response = await fetch(api, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            ...headers,
          },
          body: JSON.stringify({
            messages: messages().map((m: Message) => ({
              role: m.role,
              content: m.content,
            })),
            ...body,
          }),
          signal: abortController!.signal,
          credentials,
        });

        onResponse?.(response);

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        if (!response.body) {
          throw new Error("Response body is empty");
        }

        status.set("streaming");

        const reader = response.body.getReader();
        let accumulatedText = "";

        // Parse the stream
        for await (const part of parseStream(reader)) {
          if (part.type === "text") {
            accumulatedText += String(part.value);

            // Update assistant message reactively
            messages.set((prev: Message[]) => {
              const updated = [...prev];
              const lastIndex = updated.length - 1;
              if (lastIndex >= 0 && updated[lastIndex].role === "assistant") {
                updated[lastIndex] = {
                  ...updated[lastIndex],
                  content: accumulatedText,
                };
              }
              return updated;
            });
          } else if (part.type === "error") {
            throw new Error(String(part.value));
          }
        }

        // Finalize
        status.set("idle");
        abortController = null;

        // Get the final assistant message
        const finalMessages = messages();
        const finalAssistant = finalMessages[finalMessages.length - 1];
        if (finalAssistant && finalAssistant.role === "assistant") {
          onFinish?.(finalAssistant);
        }
      } catch (err) {
        // Handle abort
        if (err instanceof Error && err.name === "AbortError") {
          status.set("idle");
          return;
        }

        const errorInstance =
          err instanceof Error ? err : new Error(String(err));
        error.set(errorInstance);
        status.set("error");
        onError?.(errorInstance);

        // Remove the empty assistant message on error
        messages.set((prev: Message[]) => {
          const updated = [...prev];
          const lastIndex = updated.length - 1;
          if (
            lastIndex >= 0 &&
            updated[lastIndex].role === "assistant" &&
            !updated[lastIndex].content
          ) {
            updated.pop();
          }
          return updated;
        });
      }
    })();

    // Register with Suspense if enabled
    if (suspense) {
      const suspenseContext = getCurrentSuspense();
      if (suspenseContext) {
        suspenseContext.register(streamPromise);
      }
    }

    await streamPromise;
  };

  /**
   * Stop the current stream
   */
  const stop = (): void => {
    if (abortController) {
      abortController.abort();
      abortController = null;
      status.set("idle");
    }
  };

  /**
   * Reload the last assistant message
   */
  const reload = async (): Promise<void> => {
    const currentMessages = messages();

    // Find the last user message
    let lastUserIndex = -1;
    for (let i = currentMessages.length - 1; i >= 0; i--) {
      if (currentMessages[i].role === "user") {
        lastUserIndex = i;
        break;
      }
    }

    if (lastUserIndex === -1) return;

    // Remove messages from last user message onwards
    const lastUserMessage = currentMessages[lastUserIndex];
    messages.set(currentMessages.slice(0, lastUserIndex));

    // Re-submit
    await submit(lastUserMessage.content);
  };

  /**
   * Set messages directly
   */
  const setMessages = (
    newMessages: Message[] | ((prev: Message[]) => Message[])
  ): void => {
    if (typeof newMessages === "function") {
      messages.set(newMessages);
    } else {
      messages.set(newMessages);
    }
  };

  /**
   * Form submit handler
   */
  const handleSubmit = (event: Event): void => {
    event.preventDefault();
    submit();
  };

  /**
   * Input change handler
   */
  const handleInputChange = (event: Event): void => {
    const target = event.target as HTMLInputElement | HTMLTextAreaElement;
    input.set(target.value);
  };

  return {
    messages,
    input,
    status,
    error,
    isLoading,
    submit,
    stop,
    reload,
    setMessages,
    handleSubmit,
    handleInputChange,
  };
}
