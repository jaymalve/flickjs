import type { TestResult, TestStatus } from '../types.js';

export interface InternalTest {
  suite: string;
  name: string;
  fn: () => void | Promise<void>;
  skip?: boolean;
}

const results: TestResult[] = [];
let currentFile = '';
let currentSuite = '';
let testEnvironment: 'browser' | 'node' = 'browser';
let id = 0;

const beforeAllFns: (() => void | Promise<void>)[] = [];
const afterAllFns: (() => void | Promise<void>)[] = [];
const beforeEachFns: (() => void | Promise<void>)[] = [];
const afterEachFns: (() => void | Promise<void>)[] = [];
const collected: InternalTest[] = [];

export function setHarnessContext(file: string, env: 'browser' | 'node'): void {
  currentFile = file;
  testEnvironment = env;
}

export function getTestReport(): TestResult[] {
  return results;
}

export function clearHarness(): void {
  results.length = 0;
  beforeAllFns.length = 0;
  afterAllFns.length = 0;
  beforeEachFns.length = 0;
  afterEachFns.length = 0;
  collected.length = 0;
  currentSuite = '';
}

function record(
  suite: string,
  name: string,
  status: TestStatus,
  duration: number,
  error?: TestResult['error']
): void {
  results.push({
    file: currentFile,
    suite,
    test: name,
    status,
    duration,
    environment: testEnvironment,
    error
  });
}

async function runWithHooks(fn: () => void | Promise<void>): Promise<void> {
  for (const b of beforeEachFns) await b();
  try {
    await fn();
  } finally {
    for (const a of afterEachFns) await a();
  }
}

export function describe(name: string, fn: () => void): void {
  const prev = currentSuite;
  currentSuite = name;
  try {
    fn();
  } finally {
    currentSuite = prev;
  }
}

describe.skip = (_name: string, _fn: () => void) => {};

export function it(name: string, fn: () => void | Promise<void>): void {
  collected.push({ suite: currentSuite, name, fn });
}

it.skip = (name: string, _fn: () => void | Promise<void>) => {
  collected.push({ suite: currentSuite, name, fn: async () => {}, skip: true });
};

export const test = it;

export function beforeAll(fn: () => void | Promise<void>): void {
  beforeAllFns.push(fn);
}

export function afterAll(fn: () => void | Promise<void>): void {
  afterAllFns.push(fn);
}

export function beforeEach(fn: () => void | Promise<void>): void {
  beforeEachFns.push(fn);
}

export function afterEach(fn: () => void | Promise<void>): void {
  afterEachFns.push(fn);
}

class AssertionError extends Error {
  constructor(
    message: string,
    public expected?: unknown,
    public actual?: unknown
  ) {
    super(message);
    this.name = 'AssertionError';
  }
}

function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== typeof b) return false;
  if (a === null || b === null) return a === b;
  if (typeof a !== 'object') return false;
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((v, i) => deepEqual(v, b[i]));
  }
  if (Array.isArray(a) || Array.isArray(b)) return false;
  const ao = a as Record<string, unknown>;
  const bo = b as Record<string, unknown>;
  const ka = Object.keys(ao);
  const kb = Object.keys(bo);
  if (ka.length !== kb.length) return false;
  return ka.every((k) => deepEqual(ao[k], bo[k]));
}

export function createExpect(actual: unknown) {
  const chain = {
    not: {} as ReturnType<typeof createExpect>,
    toBe(expected: unknown) {
      if (!Object.is(actual, expected)) {
        throw new AssertionError(`expected ${String(expected)} (toBe), got ${String(actual)}`, expected, actual);
      }
    },
    toEqual(expected: unknown) {
      if (!deepEqual(actual, expected)) {
        throw new AssertionError(`expected deep equality`, expected, actual);
      }
    },
    toBeTruthy() {
      if (!actual) throw new AssertionError(`expected truthy, got ${String(actual)}`);
    },
    toBeFalsy() {
      if (actual) throw new AssertionError(`expected falsy, got ${String(actual)}`);
    },
    toBeNull() {
      if (actual !== null) throw new AssertionError(`expected null`, null, actual);
    },
    toBeUndefined() {
      if (actual !== undefined) throw new AssertionError(`expected undefined`, undefined, actual);
    },
    toBeDefined() {
      if (actual === undefined) throw new AssertionError(`expected defined`);
    },
    toBeGreaterThan(n: number) {
      if (typeof actual !== 'number' || actual <= n) {
        throw new AssertionError(`expected ${actual} > ${n}`);
      }
    },
    toBeLessThan(n: number) {
      if (typeof actual !== 'number' || actual >= n) {
        throw new AssertionError(`expected ${actual} < ${n}`);
      }
    },
    toContain(item: unknown) {
      if (typeof actual === 'string' && typeof item === 'string') {
        if (!actual.includes(item)) throw new AssertionError(`expected string to contain ${item}`);
      } else if (Array.isArray(actual)) {
        const ok = actual.some((x) => Object.is(x, item) || deepEqual(x, item));
        if (!ok) throw new AssertionError(`expected array to contain`, item, actual);
      } else throw new AssertionError(`toContain on unsupported type`);
    },
    toHaveLength(n: number) {
      const len = (actual as { length?: number })?.length;
      if (len !== n) throw new AssertionError(`expected length ${n}, got ${len}`);
    },
    toThrow(msg?: string | RegExp) {
      if (typeof actual !== 'function') throw new AssertionError(`toThrow expects function`);
      try {
        (actual as () => void)();
        throw new AssertionError(`expected function to throw`);
      } catch (e) {
        if (e instanceof AssertionError) throw e;
        if (msg !== undefined) {
          const m = e instanceof Error ? e.message : String(e);
          if (typeof msg === 'string' && !m.includes(msg)) {
            throw new AssertionError(`expected error message to include ${msg}`, msg, m);
          }
          if (msg instanceof RegExp && !msg.test(m)) {
            throw new AssertionError(`expected error to match ${msg}`, msg, m);
          }
        }
      }
    },
    toMatch(pattern: string | RegExp) {
      if (typeof actual !== 'string') throw new AssertionError(`toMatch expects string`);
      if (typeof pattern === 'string' && !actual.includes(pattern)) {
        throw new AssertionError(`expected match`, pattern, actual);
      }
      if (pattern instanceof RegExp && !pattern.test(actual)) {
        throw new AssertionError(`expected match`, pattern, actual);
      }
    },
    toHaveProperty(key: string, value?: unknown) {
      if (actual === null || typeof actual !== 'object') throw new AssertionError(`toHaveProperty needs object`);
      if (!(key in (actual as object))) throw new AssertionError(`missing property ${key}`);
      if (value !== undefined && !deepEqual((actual as Record<string, unknown>)[key], value)) {
        throw new AssertionError(`property ${key} value mismatch`, value, (actual as Record<string, unknown>)[key]);
      }
    }
  };

  const notChain: typeof chain = {} as typeof chain;
  for (const key of Object.keys(chain) as (keyof typeof chain)[]) {
    if (key === 'not') continue;
    const orig = chain[key] as (...args: unknown[]) => void;
    (notChain as Record<string, unknown>)[key] = (...args: unknown[]) => {
      try {
        (orig as (...a: unknown[]) => void)(...args);
      } catch {
        return;
      }
      throw new AssertionError(`expected not.${String(key)}`);
    };
  }
  chain.not = notChain;
  return chain;
}

export function expect(actual: unknown): ReturnType<typeof createExpect> {
  return createExpect(actual);
}

export async function runAllTests(): Promise<void> {
  for (const b of beforeAllFns) await b();
  try {
    for (const t of collected) {
      if (t.skip) {
        record(t.suite, t.name, 'skip', 0);
        continue;
      }
      const start = performance.now();
      try {
        await runWithHooks(t.fn);
        record(t.suite, t.name, 'pass', performance.now() - start);
      } catch (e) {
        const err = e instanceof Error ? e : new Error(String(e));
        record(t.suite, t.name, 'fail', performance.now() - start, {
          message: err.message,
          stack: err.stack ?? '',
          ...(err instanceof AssertionError ? { expected: err.expected, actual: err.actual } : {})
        });
      }
    }
  } finally {
    for (const a of afterAllFns) await a();
  }
}

/** Assign core globals for browser / node */
export function installCoreGlobals(g: typeof globalThis): void {
  const o = g as Record<string, unknown>;
  o.describe = describe;
  o.it = it;
  o.test = test;
  o.beforeAll = beforeAll;
  o.afterAll = afterAll;
  o.beforeEach = beforeEach;
  o.afterEach = afterEach;
  o.expect = expect;
}
