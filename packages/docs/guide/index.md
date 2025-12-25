# Getting Started

Flick is a tiny reactive JavaScript framework with fine-grained reactivity and no virtual DOM.

## Quick Start

### Create a New Project

```bash
# Using npx
npx create-flick-app my-app

# Or using bunx
bunx create-flick-app my-app

# Navigate and install
cd my-app
bun install

# Start development server
bun dev
```

This creates a new project with:

- Vite for development and building
- Flick runtime and compiler pre-configured
- A simple counter example to get you started

## Your First Component

```tsx
import { fx, mount } from "@flickjs/runtime";

function Counter() {
  const count = fx(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={() => count.set(count() + 1)}>+</button>
      <button onclick={() => count.set(count() - 1)}>-</button>
    </div>
  );
}

mount(Counter, document.getElementById("app"));
```

## Key Concepts

- **Fx** - Reactive values that automatically update the UI when changed
- **Run** - Side effects that run when fx change
- **Components** - Functions that return JSX
- **Fine-grained reactivity** - Only the specific DOM nodes that use an fx will update

## Next Steps

- [Installation](/guide/installation) - Manual setup for existing projects
- [Fx](/guide/fx) - Deep dive into reactive state
- [Components](/guide/components) - Building UI with components
