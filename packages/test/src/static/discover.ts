import path from 'node:path';
import fg from 'fast-glob';

export const DEFAULT_TEST_GLOBS = [
  '**/*.test.{ts,tsx,js,jsx}',
  '**/*.spec.{ts,tsx,js,jsx}',
  '**/__tests__/**/*.{ts,tsx,js,jsx}'
];

const IGNORE = [
  '**/node_modules/**',
  '**/dist/**',
  '**/build/**',
  '**/.next/**',
  '**/coverage/**',
  '**/public/**'
];

export async function discoverTests(
  rootDir: string,
  patterns: string[] = DEFAULT_TEST_GLOBS
): Promise<Set<string>> {
  const cwd = path.resolve(rootDir);
  const files = await fg(patterns, {
    cwd,
    ignore: IGNORE,
    absolute: true,
    onlyFiles: true
  });
  return new Set(files.map((f) => path.normalize(f)));
}
