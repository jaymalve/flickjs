import path from 'node:path';
import type { DiffResult } from '../git/diff.js';
import { absFromRepoRelative } from './graph.js';
import type { ModuleGraph } from './graph.js';
import type { ImpactedTest } from '../types.js';

function fileUsesServer(graph: ModuleGraph, abs: string, visited: Set<string>): boolean {
  if (visited.has(abs)) return false;
  visited.add(abs);
  const d = graph.directives.get(abs);
  if (d === 'use server') return true;
  const outs = graph.forward.get(abs);
  if (!outs) return false;
  for (const to of outs) {
    if (fileUsesServer(graph, to, visited)) return true;
  }
  return false;
}

export function environmentForTestFile(graph: ModuleGraph, testFileAbs: string): 'browser' | 'node' {
  return fileUsesServer(graph, testFileAbs, new Set()) ? 'node' : 'browser';
}

/**
 * Reverse walk: from changed files, follow importers until test entrypoints are found.
 */
export function computeImpactedTests(
  graph: ModuleGraph,
  diff: DiffResult,
  testEntrypoints: Set<string>,
  rootDir: string
): ImpactedTest[] {
  if (diff.isFullRun) {
    return [...testEntrypoints].map((f) => ({
      testFile: f,
      environment: environmentForTestFile(graph, f)
    }));
  }

  const changedAbs = new Set(
    [...diff.changedFiles].map((rel) => path.normalize(absFromRepoRelative(rootDir, rel)))
  );

  const impacted = new Set<string>();

  for (const c of changedAbs) {
    if (testEntrypoints.has(c)) impacted.add(c);
  }

  const queue = [...changedAbs];
  const seen = new Set<string>();

  while (queue.length) {
    const cur = queue.shift()!;
    if (seen.has(cur)) continue;
    seen.add(cur);

    for (const dependent of graph.getReverseDeps(cur)) {
      if (testEntrypoints.has(dependent)) impacted.add(dependent);
      if (!seen.has(dependent)) queue.push(dependent);
    }
  }

  return [...impacted].map((f) => ({
    testFile: f,
    environment: environmentForTestFile(graph, f)
  }));
}

export function filterByPattern(tests: ImpactedTest[], pattern: string | undefined): ImpactedTest[] {
  if (!pattern) return tests;
  let re: RegExp;
  try {
    re = new RegExp(pattern, 'i');
  } catch {
    re = new RegExp(pattern.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
  }
  return tests.filter((t) => re.test(t.testFile) || re.test(path.basename(t.testFile)));
}
