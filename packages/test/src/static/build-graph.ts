import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import fg from 'fast-glob';
import type { FileAnalysis } from '../types.js';
import { parseFile } from './parser.js';
import { ModuleGraph } from './graph.js';
import { AnalysisCache, type CacheEntry } from './cache.js';

const SOURCE_GLOBS = ['**/*.{ts,tsx,js,jsx,mjs,cjs}'];
const IGNORE = [
  '**/node_modules/**',
  '**/dist/**',
  '**/build/**',
  '**/.next/**',
  '**/coverage/**'
];

function analysisFromCache(abs: string, e: CacheEntry): FileAnalysis {
  return {
    filePath: abs,
    contentHash: e.contentHash,
    imports: e.imports,
    exports: e.exports,
    directive: e.directive,
    testBlocks: []
  };
}

export async function buildProjectGraph(
  rootDir: string,
  tsconfigPath: string | null,
  cache: AnalysisCache
): Promise<ModuleGraph> {
  const root = path.resolve(rootDir);
  const files = await fg(SOURCE_GLOBS, {
    cwd: root,
    ignore: IGNORE,
    absolute: true,
    onlyFiles: true
  });

  const graph = new ModuleGraph(root, tsconfigPath);
  const valid = new Set(files.map((f) => path.normalize(f)));

  for (const abs of files) {
    const norm = path.normalize(abs);
    let content: string;
    try {
      content = fs.readFileSync(norm, 'utf-8');
    } catch {
      continue;
    }
    const quickHash = cache.get(norm);
    const digest = crypto.createHash('md5').update(content).digest('hex');
    let analysis: FileAnalysis;
    if (quickHash && quickHash.contentHash === digest) {
      analysis = analysisFromCache(norm, quickHash);
    } else {
      analysis = parseFile(norm, content);
      cache.setFromAnalysis(analysis);
    }
    graph.ingestAnalysis(analysis);
  }

  cache.pruneExistingFiles(valid);
  cache.save();
  return graph;
}
