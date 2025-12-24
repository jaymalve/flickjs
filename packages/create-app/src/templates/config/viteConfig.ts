import fs from "fs-extra";
import path from "path";

export function createViteConfig(root: string) {
  fs.writeFileSync(
    path.join(root, "vite.config.js"),
    `import { defineConfig } from 'vite';
import { transformSync } from '@babel/core';
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';
import tailwindcss from '@tailwindcss/vite';
import { flickRouter } from '@flickjs/router/vite';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

// Custom Vite plugin for Flick JSX compilation
function flickPlugin() {
  const compilerPath = resolve(__dirname, 'node_modules/@flickjs/compiler/dist/index.js');
  const flickCompiler = require(compilerPath).default;

  return {
    name: 'vite-plugin-flick',
    enforce: 'pre',
    transform(code, id) {
      if (!/\\.[jt]sx$/.test(id)) return null;

      const result = transformSync(code, {
        filename: id,
        plugins: [flickCompiler],
        parserOpts: {
          plugins: ['jsx', 'typescript'],
        },
        sourceMaps: true,
        cloneInputAst: false,
        configFile: false,
        babelrc: false,
      });

      return {
        code: result.code,
        map: result.map,
      };
    },
  };
}

export default defineConfig({
  plugins: [
    flickPlugin(),
    flickRouter({ pagesDir: 'pages', root: resolve(__dirname) }),
    tailwindcss(),
  ],
});
`
  );
}
