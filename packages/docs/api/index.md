# API Reference

Flick consists of four main packages:

## Packages

### [@flickjs/runtime](/api/runtime)

The core reactive runtime. Includes:

- `fx()` - Create reactive state
- `run()` - Run side effects
- `mount()` - Mount components to the DOM
- `Suspense` - Async boundary component
- `query()` - Async data fetching
- `lazy()` - Code splitting

### [@flickjs/router](/api/router)

File-based routing for Flick. Includes:

- `Router` - Main router component
- `Link` - Navigation link component
- `navigate()` - Programmatic navigation
- `currentPath()` - Current route
- `params()` - Route params

### @flickjs/vite-plugin

Vite plugin for Flick JSX compilation. Handles all build-time transformation.

### @flickjs/compiler

Babel plugin for JSX transformation. Used internally by the vite plugin.

## Installation

```bash
# Core runtime (required)
bun add @flickjs/runtime

# Vite plugin (required for JSX)
bun add -D @flickjs/vite-plugin

# Router (optional)
bun add @flickjs/router
```
