import fs from 'node:fs';
import path from 'node:path';
import type { FileAnalysis } from '../types.js';

const CACHE_FILE = '.flick-test-cache.json';

export interface CacheEntry {
  contentHash: string;
  imports: string[];
  exports: string[];
  directive: 'use client' | 'use server' | null;
}

export class AnalysisCache {
  private root: string;
  private path: string;
  data: Map<string, CacheEntry> = new Map();

  constructor(rootDir: string) {
    this.root = path.resolve(rootDir);
    this.path = path.join(this.root, CACHE_FILE);
    this.load();
  }

  private load(): void {
    if (!fs.existsSync(this.path)) return;
    try {
      const raw = JSON.parse(fs.readFileSync(this.path, 'utf-8')) as Record<string, CacheEntry>;
      this.data = new Map(Object.entries(raw));
    } catch {
      this.data = new Map();
    }
  }

  save(): void {
    const obj = Object.fromEntries(this.data);
    fs.writeFileSync(this.path, JSON.stringify(obj, null, 2) + '\n', 'utf-8');
  }

  get(absPath: string): CacheEntry | undefined {
    return this.data.get(absPath);
  }

  setFromAnalysis(a: FileAnalysis): void {
    this.data.set(a.filePath, {
      contentHash: a.contentHash,
      imports: a.imports,
      exports: a.exports,
      directive: a.directive
    });
  }

  pruneExistingFiles(validAbsPaths: Set<string>): void {
    for (const k of this.data.keys()) {
      if (!validAbsPaths.has(k)) this.data.delete(k);
    }
  }

  /** Return true if file unchanged per hash */
  isFresh(absPath: string, contentHash: string): boolean {
    const e = this.data.get(absPath);
    return Boolean(e && e.contentHash === contentHash);
  }
}
