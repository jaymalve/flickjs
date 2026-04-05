export type TestStatus = 'pass' | 'fail' | 'skip';

export interface TestResult {
  file: string;
  suite: string;
  test: string;
  status: TestStatus;
  duration: number;
  environment: 'browser' | 'node';
  error?: {
    message: string;
    stack: string;
    expected?: unknown;
    actual?: unknown;
  };
}

export interface RunSummary {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  wallMs: number;
  browserCount: number;
  nodeCount: number;
}

export type FrameworkKind = 'nextjs' | 'react-vite' | 'react-other';

export interface ProjectInfo {
  framework: FrameworkKind;
  rootDir: string;
  tsconfigPath: string | null;
  hasServerComponents: boolean;
}

export interface FileAnalysis {
  filePath: string;
  contentHash: string;
  imports: string[];
  exports: string[];
  directive: 'use client' | 'use server' | null;
  testBlocks: { name: string; line: number; type: 'describe' | 'it' }[];
}

export interface ImpactedTest {
  testFile: string;
  environment: 'browser' | 'node';
}
