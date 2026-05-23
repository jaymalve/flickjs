# FlickJS

FlickJS is a Bun-based monorepo for a small reactive UI framework and its supporting tooling.
The core idea is simple: compile JSX to direct DOM operations and update only the nodes that
depend on changed state.

## What is in this repo

- `@flickjs/runtime`: fine-grained reactive primitives like `fx()` and `run()`
- `@flickjs/compiler`: JSX compiler for Flick components
- `@flickjs/vite-plugin`: Vite integration for the compiler
- `@flickjs/router`: file-based routing for Flick apps
- `create-flick-app`: app scaffolding CLI
- `@flickjs/react`: Flick signals and AI hooks for React
- `@flickjs/ai`: streaming and agent utilities built around the AI SDK
- `packages/docs`: VitePress documentation
- `packages/landing`: marketing site
- `packages/linter`: Flick Scan, a fast JS/TS linting project living in the same workspace

## Quick Start

Create a new app:

```bash
bun create flick-app my-app
cd my-app
bun install
bun dev
```

The generated app uses Vite, the Flick Vite plugin, the Flick router, and Tailwind.

## Small Example

```tsx
import { fx, mount } from '@flickjs/runtime';

function Counter() {
  const count = fx(0);

  return (
    <button onclick={() => count.set(count() + 1)}>
      Count: {count()}
    </button>
  );
}

mount(Counter, document.getElementById('app')!);
```

## Development

Install workspace dependencies:

```bash
bun install
```

Build the main packages:

```bash
bun run build
```

Useful commands:

- `bun run docs:dev` to run the docs site
- `bun run build:landing` to build the landing page
- `bun run clean` to remove package build output

## Project Structure

- [packages/runtime](./packages/runtime)
- [packages/compiler](./packages/compiler)
- [packages/vite-plugin](./packages/vite-plugin)
- [packages/router](./packages/router)
- [packages/create-app](./packages/create-app)
- [packages/react](./packages/react)
- [packages/ai](./packages/ai)
- [packages/docs](./packages/docs)
- [packages/landing](./packages/landing)
- [packages/linter](./packages/linter)

For more usage details, see [USAGE.md](./USAGE.md) and the docs in [packages/docs](./packages/docs).
