/**
 * Node worker harness: server tests, renderToString, Next.js API stubs.
 */
import type { ReactElement } from 'react';
import { renderToString as rts } from 'react-dom/server';
import {
  installCoreGlobals,
  clearHarness,
  runAllTests,
  getTestReport,
  setHarnessContext
} from './harness-core.js';

export class RedirectError extends Error {
  constructor(public url: string) {
    super(`redirect:${url}`);
    this.name = 'RedirectError';
  }
}

export class NotFoundError extends Error {
  constructor() {
    super('NEXT_NOT_FOUND');
    this.name = 'NotFoundError';
  }
}

export function installNodeHarness(testFile: string): void {
  clearHarness();
  setHarnessContext(testFile, 'node');
  installCoreGlobals(globalThis);
  const g = globalThis as Record<string, unknown>;
  g.renderToString = (el: ReactElement) => rts(el);
  g.cookies = () => mockCookieStore();
  g.headers = () => mockHeaders();
  g.redirect = (url: string) => {
    throw new RedirectError(url);
  };
  g.notFound = () => {
    throw new NotFoundError();
  };
}

function mockCookieStore() {
  const map = new Map<string, string>();
  return {
    get: (name: string) => ({ name, value: map.get(name) ?? '' }),
    set: (_name: string, _value: string) => {},
    getAll: () => [...map.entries()].map(([name, value]) => ({ name, value }))
  };
}

function mockHeaders() {
  const h = new Map<string, string>();
  return {
    get: (k: string) => h.get(k.toLowerCase()) ?? null,
    set: (k: string, v: string) => h.set(k.toLowerCase(), v)
  };
}

export { runAllTests, getTestReport };
