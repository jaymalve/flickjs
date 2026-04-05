#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

/** Directory containing `dist/` for `@flickjs/test`. */
const PACKAGE_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
import chokidar from 'chokidar';
import cac from 'cac';
import { detectProject } from './static/detect.js';
import { getChangedFilesFromGit } from './git/diff.js';
import { discoverTests, DEFAULT_TEST_GLOBS } from './static/discover.js';
import { buildProjectGraph } from './static/build-graph.js';
import { AnalysisCache } from './static/cache.js';
import { computeImpactedTests, filterByPattern } from './static/impact.js';
import { startTestServer } from './runtime/server.js';
import { runBrowserTests } from './runtime/browser-pool.js';
import { runNodeTests } from './runtime/node-pool.js';
import { aggregateResults, setWallTime } from './runtime/collector.js';
import { printReport } from './reporter/terminal.js';

interface RunOptions {
  cwd: string;
  base: string;
  all: boolean;
  watch: boolean;
  filter?: string;
  testMatch?: string[];
  concurrency: number;
  headed: boolean;
  verbose: boolean;
  serverOnly: boolean;
  clientOnly: boolean;
  timeout: number;
}

async function runOnce(opts: RunOptions, serverHolder: { server: Awaited<ReturnType<typeof startTestServer>>['server'] | null; url: string | null }): Promise<number> {
  const cwd = path.resolve(opts.cwd);
  const project = detectProject(cwd);
  const cache = new AnalysisCache(cwd);

  const graph = await buildProjectGraph(cwd, project.tsconfigPath, cache);

  const testGlobs = opts.testMatch?.length ? opts.testMatch : DEFAULT_TEST_GLOBS;
  const testEntrypoints = await discoverTests(cwd, testGlobs);

  const diff = opts.all ? { changedFiles: new Set<string>(), isFullRun: true } : getChangedFilesFromGit(cwd, opts.base);

  let impacted = computeImpactedTests(graph, diff, testEntrypoints, cwd);
  impacted = filterByPattern(impacted, opts.filter);

  if (opts.clientOnly) impacted = impacted.filter((i) => i.environment === 'browser');
  if (opts.serverOnly) impacted = impacted.filter((i) => i.environment === 'node');

  if (impacted.length === 0) {
    console.log(`\n${'\x1b[33m'}No tests affected by this diff.${'\x1b[0m'}\n`);
    return 0;
  }

  let server = serverHolder.server;
  let url = serverHolder.url;
  if (!server || !url) {
    const started = await startTestServer({
      rootDir: cwd,
      tsconfigPath: project.tsconfigPath,
      packageRoot: PACKAGE_ROOT
    });
    server = started.server;
    url = started.url;
    serverHolder.server = server;
    serverHolder.url = url;
  }

  const browserFiles = [...new Set(impacted.filter((i) => i.environment === 'browser').map((i) => i.testFile))];
  const nodeFiles = [...new Set(impacted.filter((i) => i.environment === 'node').map((i) => i.testFile))];

  const t0 = performance.now();
  const [browserResults, nodeResults] = await Promise.all([
    browserFiles.length
      ? runBrowserTests(url, browserFiles, {
          concurrency: opts.concurrency,
          headed: opts.headed,
          timeoutMs: opts.timeout
        })
      : Promise.resolve([]),
    nodeFiles.length ? runNodeTests(server, nodeFiles) : Promise.resolve([])
  ]);
  const wallMs = performance.now() - t0;

  const results = [...browserResults, ...nodeResults];
  let summary = aggregateResults(results);
  summary = setWallTime(summary, wallMs);

  printReport(results, summary, { verbose: opts.verbose, cwd });

  const failed = summary.failed > 0 ? 1 : 0;
  return failed;
}

async function main(): Promise<void> {
  const cli = cac('flick-test');
  cli.option('--base <ref>', 'Git ref to diff against', { default: 'HEAD' });
  cli.option('--all', 'Run all discovered tests');
  cli.option('--watch', 'Re-run on file changes');
  cli.option('--filter <pattern>', 'Filter test files / names by regex');
  cli.option('--testMatch <glob>', 'Test file glob (repeatable)', { repeatable: true });
  cli.option('--concurrency <n>', 'Parallel browser contexts', { default: 4 });
  cli.option('--headed', 'Show browser');
  cli.option('--verbose', 'Per-test timing');
  cli.option('--server-only', 'Only server tests');
  cli.option('--client-only', 'Only client tests');
  cli.option('--timeout <ms>', 'Per-test-file timeout', { default: 30_000 });

  const parsed = cli.parse(process.argv);

  const testMatchOpt = parsed.options.testMatch;
  const testMatch = testMatchOpt
    ? Array.isArray(testMatchOpt)
      ? testMatchOpt.map(String)
      : [String(testMatchOpt)]
    : undefined;

  const opts: RunOptions = {
    cwd: process.cwd(),
    base: String(parsed.options.base ?? 'HEAD'),
    all: Boolean(parsed.options.all),
    watch: Boolean(parsed.options.watch),
    filter: parsed.options.filter ? String(parsed.options.filter) : undefined,
    testMatch,
    concurrency: Number(parsed.options.concurrency ?? 4),
    headed: Boolean(parsed.options.headed),
    verbose: Boolean(parsed.options.verbose),
    serverOnly: Boolean(parsed.options.serverOnly),
    clientOnly: Boolean(parsed.options.clientOnly),
    timeout: Number(parsed.options.timeout ?? 30_000)
  };

  const holder: {
    server: Awaited<ReturnType<typeof startTestServer>>['server'] | null;
    url: string | null;
  } = { server: null, url: null };

  const exec = async (): Promise<number> => runOnce(opts, holder);

  let code = await exec();

  if (opts.watch) {
    const watchDirs = [
      'src',
      'app',
      'pages',
      'components',
      'lib',
      'test',
      '__tests__'
    ]
      .map((d) => path.join(opts.cwd, d))
      .filter((d) => fs.existsSync(d));
    const watcher = chokidar.watch(watchDirs.length ? watchDirs : opts.cwd, {
      ignored: /(^|[/\\])(node_modules|dist|\.next|coverage)[/\\]/,
      ignoreInitial: true
    });
    let timer: ReturnType<typeof setTimeout> | null = null;
    watcher.on('all', () => {
      if (timer) clearTimeout(timer);
      timer = setTimeout(async () => {
        console.log('\n\x1b[2m[watch] Re-running affected tests…\x1b[0m\n');
        code = await exec();
        process.exitCode = code;
      }, 200);
    });
    process.on('SIGINT', async () => {
      await holder.server?.close();
      process.exit(code);
    });
    return;
  }

  await holder.server?.close();
  process.exitCode = code;
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
