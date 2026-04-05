import fs from 'node:fs';
import path from 'node:path';
import type { FileAnalysis } from '../types.js';
import { loadTsConfigPaths, matchPathMappings, type TsPathMapping } from './tsconfig-paths.js';

export class ModuleGraph {
  forward = new Map<string, Set<string>>();
  reverse = new Map<string, Set<string>>();
  analyses = new Map<string, FileAnalysis>();
  /** file -> directive from analysis */
  directives = new Map<string, 'use client' | 'use server' | null>();

  private rootDir: string;
  private mapping: TsPathMapping;

  constructor(rootDir: string, tsconfigPath: string | null) {
    this.rootDir = path.resolve(rootDir);
    this.mapping = loadTsConfigPaths(this.rootDir, tsconfigPath);
  }

  clearFileEdges(fromAbs: string): void {
    const outs = this.forward.get(fromAbs);
    if (!outs) return;
    for (const to of outs) {
      const ins = this.reverse.get(to);
      if (ins) {
        ins.delete(fromAbs);
        if (ins.size === 0) this.reverse.delete(to);
      }
    }
    this.forward.delete(fromAbs);
  }

  addEdge(fromAbs: string, toAbs: string): void {
    if (!this.forward.has(fromAbs)) this.forward.set(fromAbs, new Set());
    this.forward.get(fromAbs)!.add(toAbs);
    if (!this.reverse.has(toAbs)) this.reverse.set(toAbs, new Set());
    this.reverse.get(toAbs)!.add(fromAbs);
  }

  /**
   * Resolve import specifier to absolute file path, or null if external / unsupported.
   */
  resolveImport(fromFile: string, spec: string): string | null {
    const fromDir = path.dirname(fromFile);
    let candidate: string | null = null;

    if (spec.startsWith('.') || path.isAbsolute(spec)) {
      candidate = path.resolve(fromDir, spec);
    } else {
      const mapped = matchPathMappings(spec, this.mapping);
      if (mapped) candidate = mapped;
      else {
        // bare specifier — try node_modules
        const nm = path.join(this.rootDir, 'node_modules', spec);
        if (fs.existsSync(nm) || fs.existsSync(nm + '.js')) {
          return nm;
        }
        return null;
      }
    }

    if (!candidate) return null;

    const resolved = this.probeExtensions(candidate);
    if (!resolved) return null;

    // Don't traverse into node_modules internals as project files
    const rel = path.relative(this.rootDir, resolved);
    if (rel.startsWith('node_modules')) return resolved;

    if (!fs.existsSync(resolved) || !fs.statSync(resolved).isFile()) {
      return null;
    }
    return path.normalize(resolved);
  }

  private probeExtensions(base: string): string | null {
    const candidates = [
      base,
      base + '.ts',
      base + '.tsx',
      base + '.js',
      base + '.jsx',
      base + '.mjs',
      base + '.cjs',
      path.join(base, 'index.ts'),
      path.join(base, 'index.tsx'),
      path.join(base, 'index.js'),
      path.join(base, 'index.jsx')
    ];
    for (const c of candidates) {
      if (fs.existsSync(c) && fs.statSync(c).isFile()) return c;
    }
    return null;
  }

  ingestAnalysis(analysis: FileAnalysis): void {
    const from = analysis.filePath;
    this.clearFileEdges(from);
    this.analyses.set(from, analysis);
    this.directives.set(from, analysis.directive);

    for (const spec of analysis.imports) {
      const to = this.resolveImport(from, spec);
      if (to && this.isProjectFile(to)) {
        this.addEdge(from, to);
      }
    }
  }

  isProjectFile(abs: string): boolean {
    const rel = path.relative(this.rootDir, abs);
    if (rel.startsWith('..')) return false;
    return !rel.split(path.sep).includes('node_modules');
  }

  getReverseDeps(fileAbs: string): Set<string> {
    return this.reverse.get(fileAbs) ?? new Set();
  }
}

export function absFromRepoRelative(root: string, relPosix: string): string {
  return path.normalize(path.join(root, ...relPosix.split('/')));
}
