/**
 * Browser entry: loaded by Vite in Puppeteer. Query: ?test=<absolute file path>
 */
import { installBrowserHarness, runAllTests, getTestReport } from './harness-browser.js';

const params = new URLSearchParams(window.location.search);
const testFile = params.get('test');
if (!testFile) {
  throw new Error('flick-test: missing ?test= absolute path');
}

installBrowserHarness(testFile);

const normalized = testFile.replace(/\\/g, '/');
const spec = normalized.match(/^[A-Za-z]:/)
  ? `/@fs/${normalized}`
  : `/@fs${normalized}`;

await import(/* @vite-ignore */ spec);
await runAllTests();

const report = getTestReport();
const w = window as unknown as { __flickTestReport?: (r: typeof report) => void };
if (typeof w.__flickTestReport === 'function') {
  w.__flickTestReport(report);
}
