import type { ViteDevServer } from 'vite';
import type { TestResult } from '../types.js';

function toViteFsPath(absFile: string): string {
  const normalized = absFile.replace(/\\/g, '/');
  if (/^[A-Za-z]:/.test(normalized)) {
    return '/@fs/' + normalized;
  }
  return '/@fs' + normalized;
}

export async function runNodeTests(
  server: ViteDevServer,
  testFiles: string[]
): Promise<TestResult[]> {
  const out: TestResult[] = [];
  const { installNodeHarness, runAllTests, getTestReport } = await import('./harness-node.js');

  for (const file of testFiles) {
    installNodeHarness(file);
    try {
      await server.ssrLoadModule(toViteFsPath(file));
      await runAllTests();
      out.push(...getTestReport());
    } catch (e) {
      out.push({
        file,
        suite: '',
        test: '__file__',
        status: 'fail',
        duration: 0,
        environment: 'node',
        error: {
          message: e instanceof Error ? e.message : String(e),
          stack: e instanceof Error ? e.stack ?? '' : ''
        }
      });
    }
  }
  return out;
}
