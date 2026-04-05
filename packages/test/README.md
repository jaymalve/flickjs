# @flickjs/test

Diff-first test runner for **React** and **Next.js**: uses your git diff and an AST-built module graph to run only impacted tests. Client tests execute in a real browser (Puppeteer); server-bound tests run in Node via Vite SSR.

## Install

```bash
npm install -D @flickjs/test react react-dom
```

## Usage

From your project root:

```bash
npx flick-test
```

- **`--base <ref>`** — git ref to diff against (default: `HEAD`)
- **`--all`** — run every discovered test
- **`--watch`** — re-run on file changes
- **`--filter <regex>`** — filter by file / name
- **`--concurrency <n>`** — parallel browser pages (default: `4`)
- **`--headed`** — show the browser
- **`--server-only` / `--client-only`** — environment filter
- **`--timeout <ms>`** — per file (default: `30000`)

Test files match `**/*.test.*`, `**/*.spec.*`, and `**/__tests__/**` by default.

## Authoring tests

Use `describe`, `it`, `expect`, and (in the browser) `render`, `screen`, `fireEvent`, `waitFor`, and `act` — see the runtime harness in `src/runtime/harness-browser.ts`.
