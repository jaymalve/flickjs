import fs from 'node:fs';
import path from 'node:path';
import type { FrameworkKind, ProjectInfo } from '../types.js';

function exists(p: string): boolean {
  try {
    fs.accessSync(p);
    return true;
  } catch {
    return false;
  }
}

function readJson(p: string): Record<string, unknown> | null {
  try {
    return JSON.parse(fs.readFileSync(p, 'utf-8')) as Record<string, unknown>;
  } catch {
    return null;
  }
}

function hasNextConfig(root: string): boolean {
  const names = ['next.config.js', 'next.config.mjs', 'next.config.ts', 'next.config.cjs'];
  return names.some((n) => exists(path.join(root, n)));
}

function packageHasDep(pkg: Record<string, unknown> | null, name: string): boolean {
  if (!pkg) return false;
  const deps = pkg.dependencies as Record<string, string> | undefined;
  const dev = pkg.devDependencies as Record<string, string> | undefined;
  return Boolean(deps?.[name] || dev?.[name]);
}

function findTsconfig(root: string): string | null {
  const p = path.join(root, 'tsconfig.json');
  return exists(p) ? p : null;
}

function viteConfigUsesReact(root: string): boolean {
  const names = ['vite.config.ts', 'vite.config.js', 'vite.config.mjs', 'vite.config.cjs'];
  for (const n of names) {
    const p = path.join(root, n);
    if (!exists(p)) continue;
    try {
      const s = fs.readFileSync(p, 'utf-8');
      if (/@vitejs\/plugin-react|plugin-react/.test(s)) return true;
    } catch {
      /* ignore */
    }
  }
  return false;
}

function detectFramework(root: string): FrameworkKind {
  const pkgPath = path.join(root, 'package.json');
  const pkg = readJson(pkgPath);

  if (hasNextConfig(root) || packageHasDep(pkg, 'next')) {
    return 'nextjs';
  }
  const hasViteConfig =
    exists(path.join(root, 'vite.config.ts')) ||
    exists(path.join(root, 'vite.config.js')) ||
    exists(path.join(root, 'vite.config.mjs'));
  if (hasViteConfig && (viteConfigUsesReact(root) || packageHasDep(pkg, 'vite'))) {
    if (packageHasDep(pkg, 'react')) return 'react-vite';
  }
  if (packageHasDep(pkg, 'react')) {
    return 'react-other';
  }
  return 'react-vite';
}

export function detectProject(rootDir: string): ProjectInfo {
  const framework = detectFramework(rootDir);
  const tsconfigPath = findTsconfig(rootDir);
  return {
    framework,
    rootDir: path.resolve(rootDir),
    tsconfigPath,
    hasServerComponents: framework === 'nextjs'
  };
}
