type AiClientKind = 'chat' | 'object';

const AI_CLIENT_LOG_PREFIX = '[flickjs/ai:client]';

function headerCount(headers?: HeadersInit): number {
  if (!headers) return 0;

  if (typeof Headers !== 'undefined' && headers instanceof Headers) {
    return Array.from(headers.keys()).length;
  }

  if (Array.isArray(headers)) {
    return headers.length;
  }

  return Object.keys(headers).length;
}

export function summarizeHeaders(headers?: HeadersInit): { headerCount: number } {
  return { headerCount: headerCount(headers) };
}

export function summarizeBody(
  body?: Record<string, unknown>
): { bodyKeys: string[]; bodyKeyCount: number } {
  const bodyKeys = body ? Object.keys(body).sort() : [];
  return {
    bodyKeys,
    bodyKeyCount: bodyKeys.length
  };
}

export function summarizeJsonPayload(
  json: string
): { payloadBytes: number; payloadKeys: string[]; payloadKeyCount: number } {
  const parsed = JSON.parse(json) as Record<string, unknown>;
  const payloadKeys = Object.keys(parsed).sort();

  return {
    payloadBytes: json.length,
    payloadKeys,
    payloadKeyCount: payloadKeys.length
  };
}

export function createAiClientLogger(client: AiClientKind) {
  const log = (method: 'debug' | 'error', event: string, details: Record<string, unknown>): void => {
    console[method](AI_CLIENT_LOG_PREFIX, {
      client,
      event,
      ...details
    });
  };

  return {
    debug(event: string, details: Record<string, unknown>): void {
      log('debug', event, details);
    },
    error(event: string, error: Error, details: Record<string, unknown>): void {
      console.error(
        AI_CLIENT_LOG_PREFIX,
        {
          client,
          event,
          error: {
            name: error.name,
            message: error.message
          },
          ...details
        },
        error
      );
    }
  };
}
