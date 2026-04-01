import { useRef, useCallback } from 'react';
import { aiChat } from '@flickjs/ai';
import type { AiChatOptions, Message } from '@flickjs/ai';
import { useFxValue } from '../internal/use-fx-value';

export interface UseAiChatReturn {
  messages: Message[];
  input: string;
  setInput: (value: string) => void;
  isLoading: boolean;
  error: Error | undefined;
  submit: (message?: string) => Promise<void>;
  stop: () => void;
  reload: () => Promise<void>;
  setMessages: (msgs: Message[] | ((prev: Message[]) => Message[])) => void;
  handleSubmit: (e: React.FormEvent) => void;
  handleInputChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => void;
}

/**
 * React hook for AI chat with streaming support.
 * Wraps FlickJS's aiChat() with React-compatible state management.
 *
 * @example
 * ```tsx
 * import { useAiChat } from '@flickjs/react/ai'
 *
 * function Chat() {
 *   const { messages, input, handleInputChange, handleSubmit, isLoading } =
 *     useAiChat({ api: '/api/chat' })
 *
 *   return (
 *     <div>
 *       {messages.map(m => <div key={m.id}>{m.content}</div>)}
 *       <form onSubmit={handleSubmit}>
 *         <input value={input} onChange={handleInputChange} />
 *         <button type="submit" disabled={isLoading}>Send</button>
 *       </form>
 *     </div>
 *   )
 * }
 * ```
 */
export function useAiChat(options: Omit<AiChatOptions, 'suspense'>): UseAiChatReturn {
  const chatRef = useRef<ReturnType<typeof aiChat> | null>(null);
  if (chatRef.current === null) {
    chatRef.current = aiChat({ ...options, suspense: false });
  }
  const chat = chatRef.current;

  const messages = useFxValue(chat.messages);
  const input = useFxValue(chat.input);
  const status = useFxValue(chat.status);
  const error = useFxValue(chat.error);

  const isLoading = status === 'submitting' || status === 'streaming';

  const setInput = useCallback((value: string) => chat.input.set(value), [chat]);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      chat.submit();
    },
    [chat]
  );

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
      chat.input.set(e.target.value);
    },
    [chat]
  );

  return {
    messages,
    input,
    setInput,
    isLoading,
    error,
    submit: chat.submit,
    stop: chat.stop,
    reload: chat.reload,
    setMessages: chat.setMessages,
    handleSubmit,
    handleInputChange
  };
}
