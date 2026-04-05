import crypto from 'node:crypto';
import fs from 'node:fs';
import { parse } from '@babel/parser';
import traverseLib from '@babel/traverse';
const traverse = ((traverseLib as unknown as { default?: typeof traverseLib }).default ??
  traverseLib) as typeof import('@babel/traverse').default;
import * as t from '@babel/types';
import type { FileAnalysis } from '../types.js';

const PARSER_OPTS = {
  sourceType: 'module' as const,
  allowImportExportEverywhere: true,
  plugins: [
    'jsx',
    'typescript',
    'decorators-legacy',
    'importAssertions' as const
  ]
};

function hashContent(content: string): string {
  return crypto.createHash('md5').update(content).digest('hex');
}

function firstArgString(call: t.CallExpression): string | null {
  const a0 = call.arguments[0];
  if (!a0) return null;
  if (t.isStringLiteral(a0)) return a0.value;
  if (t.isTemplateLiteral(a0) && a0.quasis.length === 1 && !a0.expressions.length) {
    return a0.quasis[0].value.cooked ?? a0.quasis[0].value.raw;
  }
  return null;
}

function isDescribeCallee(callee: t.Expression | t.V8IntrinsicIdentifier): boolean {
  if (t.isIdentifier(callee) && (callee.name === 'describe' || callee.name === 'context')) return true;
  if (t.isMemberExpression(callee) && !callee.computed) {
    const prop = callee.property;
    if (t.isIdentifier(prop) && prop.name === 'describe' && t.isIdentifier(callee.object))
      return callee.object.name === 'test';
  }
  return false;
}

function isItCallee(callee: t.Expression | t.V8IntrinsicIdentifier): boolean {
  if (t.isIdentifier(callee) && (callee.name === 'it' || callee.name === 'test')) return true;
  return false;
}

export function parseFile(filePath: string, content: string): FileAnalysis {
  const contentHash = hashContent(content);
  const imports: string[] = [];
  const exports: string[] = [];
  let directive: FileAnalysis['directive'] = null;
  const testBlocks: FileAnalysis['testBlocks'] = [];

  let ast;
  try {
    ast = parse(content, { ...PARSER_OPTS, filename: filePath });
  } catch {
    return {
      filePath,
      contentHash,
      imports: [],
      exports: [],
      directive: null,
      testBlocks: []
    };
  }

  const body0 = ast.program.body[0];
  if (t.isExpressionStatement(body0) && t.isStringLiteral(body0.expression)) {
    if (body0.expression.value === 'use client') directive = 'use client';
    if (body0.expression.value === 'use server') directive = 'use server';
  }

  traverse(ast, {
    ImportDeclaration(path) {
      imports.push(path.node.source.value);
    },
    CallExpression(path) {
      const { callee, arguments: args } = path.node;
      if (t.isImport(callee) && args[0] && t.isStringLiteral(args[0])) {
        imports.push(args[0].value);
      }
      if (isDescribeCallee(callee)) {
        const name = firstArgString(path.node);
        if (name) testBlocks.push({ name, line: path.node.loc?.start.line ?? 0, type: 'describe' });
      } else if (isItCallee(callee)) {
        const name = firstArgString(path.node);
        if (name) testBlocks.push({ name, line: path.node.loc?.start.line ?? 0, type: 'it' });
      }
    },
    ExportNamedDeclaration(path) {
      const decl = path.node.declaration;
      if (t.isFunctionDeclaration(decl) && decl.id) exports.push(decl.id.name);
      else if (t.isVariableDeclaration(decl)) {
        for (const d of decl.declarations) {
          if (t.isIdentifier(d.id)) exports.push(d.id.name);
        }
      }
      for (const spec of path.node.specifiers) {
        if (t.isExportSpecifier(spec) && t.isIdentifier(spec.exported)) {
          exports.push(spec.exported.name);
        }
      }
    },
    ExportDefaultDeclaration(path) {
      exports.push('default');
    }
  });

  return { filePath, contentHash, imports, exports, directive, testBlocks };
}

export function parseFileFromDisk(absPath: string): FileAnalysis {
  const content = fs.readFileSync(absPath, 'utf-8');
  return parseFile(absPath, content);
}
