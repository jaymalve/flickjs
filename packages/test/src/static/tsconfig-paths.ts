import fs from 'node:fs';
import path from 'node:path';

export interface TsPathMapping {
  baseUrl: string;
  paths: Map<string, string[]>;
}

export function loadTsConfigPaths(rootDir: string, tsconfigPath: string | null): TsPathMapping {
  const result: TsPathMapping = {
    baseUrl: path.resolve(rootDir),
    paths: new Map()
  };
  if (!tsconfigPath || !fs.existsSync(tsconfigPath)) return result;

  try {
    const raw = JSON.parse(fs.readFileSync(tsconfigPath, 'utf-8')) as {
      compilerOptions?: { baseUrl?: string; paths?: Record<string, string[]> };
    };
    const co = raw.compilerOptions;
    const dir = path.dirname(tsconfigPath);
    if (co?.baseUrl) {
      result.baseUrl = path.resolve(dir, co.baseUrl);
    } else {
      result.baseUrl = dir;
    }
    if (co?.paths) {
      for (const [k, vals] of Object.entries(co.paths)) {
        result.paths.set(k, vals);
      }
    }
  } catch {
    /* ignore */
  }
  return result;
}

/**
 * Map TS path alias import (e.g. `@/components/x`) to absolute filesystem path.
 */
export function matchPathMappings(spec: string, mapping: TsPathMapping): string | null {
  for (const [pattern, targets] of mapping.paths) {
    if (pattern.endsWith('/*')) {
      const prefix = pattern.slice(0, -2);
      if (spec === prefix) {
        for (const tp of targets) {
          if (tp.endsWith('/*')) {
            const base = tp.slice(0, -2).replace(/^\.\//, '');
            return path.join(mapping.baseUrl, base);
          }
        }
        continue;
      }
      if (!spec.startsWith(prefix + '/')) continue;
      const rest = spec.slice(prefix.length + 1);
      for (const tp of targets) {
        if (tp.endsWith('/*')) {
          const base = tp.slice(0, -2).replace(/^\.\//, '');
          return path.join(mapping.baseUrl, base, rest);
        }
        const joined = path.join(mapping.baseUrl, tp.replace(/^\.\//, ''), rest);
        return joined;
      }
    } else if (spec === pattern) {
      for (const tp of targets) {
        return path.join(mapping.baseUrl, tp.replace(/^\.\//, ''));
      }
    }
  }
  return null;
}
