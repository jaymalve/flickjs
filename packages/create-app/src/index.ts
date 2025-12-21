#!/usr/bin/env node
import fs from "fs-extra";
import path from "path";

const name = process.argv[2] || "flick-app";
const root = path.join(process.cwd(), name);

fs.ensureDirSync(root);

// Create package.json
fs.writeFileSync(
  path.join(root, "package.json"),
  JSON.stringify(
    {
      name: name,
      version: "0.1.0",
      type: "module",
      scripts: {
        dev: "bunx --bun vite",
        build: "bunx --bun vite build",
        preview: "bunx --bun vite preview",
      },
      dependencies: {
        "@flickjs/runtime": "^0.0.1-beta.1",
      },
      devDependencies: {
        "@babel/core": "^7.24.0",
        "@flickjs/compiler": "^0.0.1-beta.1",
        vite: "^5.0.0",
      },
    },
    null,
    2
  ) + "\n"
);

// Create .gitignore
fs.writeFileSync(
  path.join(root, ".gitignore"),
  `node_modules/
dist/
.DS_Store
*.log
`
);

// Create babel.config.js
fs.writeFileSync(
  path.join(root, "babel.config.js"),
  `export default {
  plugins: ["@flickjs/compiler"]
};
`
);

// Create vite.config.js
fs.writeFileSync(
  path.join(root, "vite.config.js"),
  `import { defineConfig } from 'vite';
import { transformSync } from '@babel/core';
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

// Custom Vite plugin for Flick JSX compilation
function flickPlugin() {
  // Load the compiler using require to get the actual module
  const compilerPath = resolve(__dirname, 'node_modules/@flickjs/compiler/dist/index.js');
  const flickCompiler = require(compilerPath).default;

  return {
    name: 'vite-plugin-flick',
    transform(code, id) {
      if (!/\\.[jt]sx$/.test(id)) return null;

      const result = transformSync(code, {
        filename: id,
        plugins: [flickCompiler],
        sourceMaps: true,
      });

      return {
        code: result.code,
        map: result.map
      };
    }
  };
}

export default defineConfig({
  plugins: [flickPlugin()]
});
`
);

// Create tsconfig.json (optional but helpful)
fs.writeFileSync(
  path.join(root, "tsconfig.json"),
  JSON.stringify(
    {
      compilerOptions: {
        target: "ES2020",
        module: "ESNext",
        moduleResolution: "bundler",
        jsx: "preserve",
        strict: true,
        esModuleInterop: true,
        skipLibCheck: true,
        types: ["bun-types"],
      },
      include: ["src/**/*"],
    },
    null,
    2
  ) + "\n"
);

// Create index.html
fs.writeFileSync(
  path.join(root, "index.html"),
  `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Flick App</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
`
);

fs.ensureDirSync(path.join(root, "src"));

// Create src/main.tsx
fs.writeFileSync(
  path.join(root, "src/main.tsx"),
  `import { signal, mount } from "@flickjs/runtime"

function App() {
  const count = signal(0)

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={() => count.set(count() + 1)}>
        Increment
      </button>
      <button onclick={() => count.set(count() - 1)}>
        Decrement
      </button>
      <button onclick={() => count.set(0)}>
        Reset
      </button>
    </div>
  )
}

mount(App, document.getElementById("app"))
`
);

console.log("Flick app created!");
console.log("");
console.log("Next steps:");
console.log(`  cd ${name}`);
console.log("  bun install");
console.log("  bun dev");
