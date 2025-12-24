# API Reference

Flick consists of three main packages:

## Packages

### [@flickjs/runtime](/api/runtime)

The core reactive runtime. Includes:

- `signal()` - Create reactive state
- `effect()` - Run side effects
- `mount()` - Mount components to the DOM
- `Suspense` - Async boundary component
- `resource()` - Async data fetching
- `lazy()` - Code splitting

### [@flickjs/router](/api/router)

File-based routing for Flick. Includes:

- `Router` - Main router component
- `Link` - Navigation link component
- `navigate()` - Programmatic navigation
- `currentPath()` - Current route 
- `params()` - Route params

### [@flickjs/compiler](/api/suspense)

Babel plugin for JSX transformation. Used at build time only.

## Installation

```bash
# Core runtime (required)
bun add @flickjs/runtime

# Compiler (required for JSX)
bun add -D @flickjs/compiler @babel/core

# Router (optional)
bun add @flickjs/router
```
