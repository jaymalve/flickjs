import { aiChat as createAiChat } from './chat';
import { aiObject as createAiObject } from './object';
import type {
  Message,
  ChatStatus,
  AiChatOptions,
  AiChat,
  AiObjectOptions,
  AiObject
} from './types';
import { createAiClientLogger, summarizeBody, summarizeHeaders } from './logger';

const chatLogger = createAiClientLogger('chat');
const objectLogger = createAiClientLogger('object');

export function aiChat(options: AiChatOptions): AiChat {
  const context = {
    api: options.api,
    suspense: options.suspense ?? false,
    initialMessageCount: options.initialMessages?.length ?? 0,
    initialInputLength: options.initialInput?.length ?? 0,
    hasCredentials: options.credentials !== undefined,
    ...summarizeHeaders(options.headers),
    ...summarizeBody(options.body)
  };

  chatLogger.debug('create', context);
  const result = createAiChat(options);
  chatLogger.debug('create:success', context);
  return result;
}

export function aiObject<T>(options: AiObjectOptions<T>): AiObject<T> {
  const context = {
    api: options.api,
    suspense: options.suspense ?? false,
    hasCredentials: options.credentials !== undefined,
    ...summarizeHeaders(options.headers),
    ...summarizeBody(options.body)
  };

  objectLogger.debug('create', context);
  const result = createAiObject(options);
  objectLogger.debug('create:success', context);
  return result;
}

export type { Message, ChatStatus, AiChatOptions, AiChat, AiObjectOptions, AiObject };
