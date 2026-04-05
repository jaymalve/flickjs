import type { TestResult, RunSummary } from '../types.js';

export function aggregateResults(results: TestResult[]): RunSummary {
  const passed = results.filter((r) => r.status === 'pass').length;
  const failed = results.filter((r) => r.status === 'fail').length;
  const skipped = results.filter((r) => r.status === 'skip').length;
  const browserCount = results.filter((r) => r.environment === 'browser').length;
  const nodeCount = results.filter((r) => r.environment === 'node').length;
  return {
    total: results.length,
    passed,
    failed,
    skipped,
    wallMs: 0,
    browserCount,
    nodeCount
  };
}

export function setWallTime(summary: RunSummary, ms: number): RunSummary {
  return { ...summary, wallMs: ms };
}

/** Map stack lines through a source-map consumer when available (optional enhancement). */
export async function mapStack(stack: string, _consumer: unknown): Promise<string> {
  return stack;
}
