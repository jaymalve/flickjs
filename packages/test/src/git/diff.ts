import { execSync } from 'node:child_process';
import path from 'node:path';

export interface DiffResult {
  /** Repo-relative normalized paths */
  changedFiles: Set<string>;
  isFullRun: boolean;
}

const CONFIG_BASENAMES = new Set([
  'tsconfig.json',
  'package.json',
  'tailwind.config.js',
  'tailwind.config.ts',
  'tailwind.config.mjs',
  'postcss.config.js',
  'postcss.config.mjs'
]);

function isConfigPath(rel: string): boolean {
  const base = path.basename(rel);
  if (CONFIG_BASENAMES.has(base)) return true;
  if (base.startsWith('next.config')) return true;
  if (base.startsWith('vite.config')) return true;
  return false;
}

/**
 * Paths that participate in the module graph (relative to repo root, posix separators).
 */
export function isSourceLike(rel: string): boolean {
  if (!rel) return false;
  const lower = rel.toLowerCase();
  const skip =
    lower.includes('node_modules') ||
    lower.startsWith('dist/') ||
    lower.startsWith('build/') ||
    lower.startsWith('.next/') ||
    lower.startsWith('coverage/');
  if (skip) return false;
  return /\.(mjsx|mtsx|jsx|tsx|js|ts|cjs|mjs)$/.test(rel);
}

export function getChangedFilesFromGit(cwd: string, base = 'HEAD'): DiffResult {
  let output: string;
  try {
    output = execSync(`git diff --name-only --diff-filter=ACMR ${base}`, {
      cwd,
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
  } catch {
    // Not a git repo or git error — treat as full run with no changed files
    return { changedFiles: new Set(), isFullRun: true };
  }

  const lines = output
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean);

  const changedFiles = new Set<string>();
  let isFullRun = false;

  for (const line of lines) {
    const rel = line.replace(/\\/g, '/');
    if (isConfigPath(rel)) {
      isFullRun = true;
      continue;
    }
    // non-source changes don't affect graph but we might still want to know
    if (isSourceLike(rel)) {
      changedFiles.add(rel);
    }
  }

  return { changedFiles, isFullRun };
}
