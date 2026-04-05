import puppeteer from 'puppeteer';
import type { TestResult } from '../types.js';

export async function runBrowserTests(
  baseUrl: string,
  testFiles: string[],
  opts: { concurrency: number; headed: boolean; timeoutMs: number }
): Promise<TestResult[]> {
  if (testFiles.length === 0) return [];

  const browser = await puppeteer.launch({
    headless: opts.headed ? false : true,
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
  });

  const out: TestResult[] = [];
  let nextIndex = 0;

  async function runWorker(): Promise<void> {
    for (;;) {
      const i = nextIndex++;
      if (i >= testFiles.length) return;
      const file = testFiles[i]!;
      const page = await browser.newPage();
      try {
        let settled = false;
        let resolveReport!: (r: TestResult[]) => void;
        const reportPromise = new Promise<TestResult[]>((res) => {
          resolveReport = res;
        });
        const done = (r: TestResult[]) => {
          if (settled) return;
          settled = true;
          clearTimeout(failTimer);
          resolveReport(r);
        };
        const failTimer = setTimeout(() => {
          done([
            {
              file,
              suite: '',
              test: '__file__',
              status: 'fail',
              duration: 0,
              environment: 'browser',
              error: {
                message: `Timeout after ${opts.timeoutMs}ms`,
                stack: ''
              }
            }
          ]);
        }, opts.timeoutMs);

        await page.exposeFunction('__flickTestReport', (r: TestResult[]) => {
          done(r);
        });

        const url = `${baseUrl}/__flick_harness__?test=${encodeURIComponent(file)}`;
        await page.goto(url, {
          waitUntil: 'load',
          timeout: opts.timeoutMs
        });

        const r = await reportPromise;
        out.push(...r);
      } catch (e) {
        out.push({
          file,
          suite: '',
          test: '__file__',
          status: 'fail',
          duration: 0,
          environment: 'browser',
          error: {
            message: e instanceof Error ? e.message : String(e),
            stack: e instanceof Error ? e.stack ?? '' : ''
          }
        });
      } finally {
        await page.close().catch(() => {});
      }
    }
  }

  const n = Math.max(1, Math.min(opts.concurrency, testFiles.length));
  await Promise.all(Array.from({ length: n }, () => runWorker()));
  await browser.close();
  return out;
}
