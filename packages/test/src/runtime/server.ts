import fs from 'node:fs';
import path from 'node:path';
import react from '@vitejs/plugin-react';
import type { Plugin, ViteDevServer } from 'vite';
import { createServer } from 'vite';
import { loadTsConfigPaths } from '../static/tsconfig-paths.js';

function viteAliasesFromTsconfig(tsconfigPath: string | null, root: string): Record<string, string> {
  const aliases: Record<string, string> = {};
  if (!tsconfigPath || !fs.existsSync(tsconfigPath)) return aliases;
  const mapping = loadTsConfigPaths(root, tsconfigPath);
  for (const [pat, targets] of mapping.paths) {
    if (!targets[0]) continue;
    const key = pat.endsWith('/*') ? pat.slice(0, -2) : pat;
    const t0 = targets[0];
    if (t0.endsWith('/*')) {
      const base = t0.slice(0, -2).replace(/^\.\//, '');
      aliases[key] = path.join(mapping.baseUrl, base);
    } else {
      aliases[key] = path.join(mapping.baseUrl, t0.replace(/^\.\//, ''));
    }
  }
  return aliases;
}

function flickHarnessPlugin(packageRoot: string): Plugin {
  return {
    name: 'flick-test-harness',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (!req.url?.startsWith('/__flick_harness__')) {
          next();
          return;
        }
        const u = new URL(req.url, 'http://localhost');
        const test = u.searchParams.get('test');
        if (!test) {
          res.statusCode = 400;
          res.end('missing test param');
          return;
        }
        const pkgRoot = packageRoot;
        const bootJs = path.join(pkgRoot, 'dist', 'harness-boot.js');
        const bootTs = path.join(pkgRoot, 'src', 'runtime', 'harness-boot.ts');
        const bootAbs = (fs.existsSync(bootJs) ? bootJs : bootTs).replace(/\\/g, '/');
        const bootSrc =
          /^[A-Za-z]:/.test(bootAbs) ? `/@fs/${bootAbs}` : `/@fs${bootAbs}`;
        const html = `<!doctype html><html lang="en"><head><meta charset="UTF-8"/></head><body>
<script type="module" src="${bootSrc}"></script>
</body></html>`;
        res.setHeader('Content-Type', 'text/html; charset=utf-8');
        res.end(html);
      });
    }
  };
}

export interface StartTestServerOptions {
  rootDir: string;
  tsconfigPath: string | null;
  /** Root of `@flickjs/test` install (folder that contains `dist/`). */
  packageRoot: string;
}

export async function startTestServer(opts: StartTestServerOptions): Promise<{
  server: ViteDevServer;
  port: number;
  url: string;
}> {
  const rootDir = path.resolve(opts.rootDir);
  const server = await createServer({
    root: rootDir,
    plugins: [react(), flickHarnessPlugin(opts.packageRoot)],
    server: {
      port: 0,
      strictPort: false,
      hmr: false
    },
    define: {
      'process.env.NODE_ENV': JSON.stringify('test')
    },
    resolve: {
      alias: viteAliasesFromTsconfig(opts.tsconfigPath, rootDir)
    },
    optimizeDeps: {
      include: ['react', 'react-dom', 'react-dom/client', 'react/jsx-runtime']
    },
    ssr: {
      noExternal: ['react', 'react-dom']
    }
  });

  server.config.server.fs = {
    ...server.config.server.fs,
    allow: [...(server.config.server.fs?.allow ?? []), opts.packageRoot, rootDir]
  };

  await server.listen();
  const info = server.httpServer?.address();
  const port = typeof info === 'object' && info && 'port' in info ? (info as { port: number }).port : 5173;
  const url = `http://127.0.0.1:${port}`;
  return { server, port, url };
}
