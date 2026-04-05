import path from 'node:path';
import type { TestResult, RunSummary } from '../types.js';

const G = '\x1b[32m';
const R = '\x1b[31m';
const Y = '\x1b[33m';
const D = '\x1b[2m';
const B = '\x1b[1m';
const X = '\x1b[0m';

export function printReport(
  results: TestResult[],
  summary: RunSummary,
  opts: { verbose: boolean; cwd: string }
): void {
  const byFile = new Map<string, TestResult[]>();
  for (const r of results) {
    if (!byFile.has(r.file)) byFile.set(r.file, []);
    byFile.get(r.file)!.push(r);
  }

  console.log(`\n${B}flick-test${X} ${D}(${summary.browserCount} browser / ${summary.nodeCount} node results)${X}\n`);

  for (const [file, tests] of byFile) {
    const rel = path.relative(opts.cwd, file) || file;
    console.log(`${D}▸${X} ${rel}`);
    for (const t of tests) {
      const sym =
        t.status === 'pass' ? `${G}✓${X}` : t.status === 'skip' ? `${Y}○${X}` : `${R}✗${X}`;
      const name = [t.suite, t.test].filter(Boolean).join(' ');
      const timing = opts.verbose ? ` ${D}${t.duration.toFixed(1)}ms${X}` : '';
      console.log(`  ${sym} ${name}${timing}`);
      if (t.status === 'fail' && t.error) {
        console.log(`    ${R}${t.error.message}${X}`);
        if (t.error.stack) {
           const lines = t.error.stack.split('\n').slice(0, 8).join('\n');
           console.log(`${D}${lines}${X}`);
        }
      }
    }
  }

  const line = [
    summary.failed > 0 ? `${R}${summary.failed} failed${X}` : '',
    `${G}${summary.passed} passed${X}`,
    summary.skipped > 0 ? `${Y}${summary.skipped} skipped${X}` : ''
  ]
    .filter(Boolean)
    .join('  ');

  console.log(`\n${line}  ${D}(${summary.total} total in ${summary.wallMs.toFixed(0)}ms)${X}\n`);
}
