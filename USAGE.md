# Flick Usage Guide

A tiny reactive JS framework from the future

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

---

## Manual Installation

If you want to add Flick to an existing project:

```bash
# Install runtime (required)
bun add @flickjs/runtime

# Install compiler (required for JSX)
bun add -D @flickjs/compiler @babel/core

# Install router (optional)
bun add @flickjs/router
```

### Vite Configuration

Create or update `vite.config.js`:

```js
import { defineConfig } from "vite";
import { transformSync } from "@babel/core";
import { createRequire } from "module";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

function flickPlugin() {
  const compilerPath = resolve(
    __dirname,
    "node_modules/@flickjs/compiler/dist/index.js"
  );
  const flickCompiler = require(compilerPath).default;

  return {
    name: "vite-plugin-flick",
    transform(code, id) {
      if (!/\.[jt]sx$/.test(id)) return null;

      const result = transformSync(code, {
        filename: id,
        plugins: [flickCompiler],
        sourceMaps: true,
      });

      return {
        code: result.code,
        map: result.map,
      };
    },
  };
}

export default defineConfig({
  plugins: [flickPlugin()],
});
```

---

## Core Concepts

### Signals

Signals are reactive values that automatically update the UI when changed.

```tsx
import { signal, mount } from "@flickjs/runtime";

function Counter() {
  const count = signal(0);

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

**Key points:**

- `signal(initialValue)` creates a reactive signal
- `count()` reads the current value
- `count.set(newValue)` updates the value and triggers UI updates
- Only the specific DOM nodes that use the signal will update (fine-grained reactivity)

### Derived Values

Computed values that automatically update when their dependencies change:

```tsx
function PriceCalculator() {
  const price = signal(100);
  const quantity = signal(1);

  // Derived value - automatically updates
  const total = () => price() * quantity();

  return (
    <div>
      <p>Price: ${price()}</p>
      <p>Quantity: {quantity()}</p>
      <p>Total: ${total()}</p>
      <button onclick={() => quantity.set(quantity() + 1)}>Add one</button>
    </div>
  );
}
```

### Effects

Run side effects when signals change:

```tsx
import { signal, effect, mount } from "@flickjs/runtime";

function Logger() {
  const count = signal(0);

  // Runs whenever count changes
  effect(() => {
    console.log("Count changed to:", count());
  });

  return (
    <button onclick={() => count.set(count() + 1)}>Click me ({count()})</button>
  );
}
```

---

## Components

### Function Components

Components are just functions that return JSX:

```tsx
function Greeting({ name }) {
  return <h1>Hello, {name}!</h1>;
}

function App() {
  return (
    <div>
      <Greeting name="World" />
      <Greeting name="Flick" />
    </div>
  );
}
```

### Passing Children

```tsx
function Card({ children, title }) {
  return (
    <div class="card">
      <h2>{title}</h2>
      <div class="card-body">{children}</div>
    </div>
  );
}

function App() {
  return (
    <Card title="Welcome">
      <p>This is the card content.</p>
    </Card>
  );
}
```

### Conditional Rendering

```tsx
function Toggle() {
  const visible = signal(true);

  return (
    <div>
      <button onclick={() => visible.set(!visible())}>Toggle</button>
      {visible() && <p>I'm visible!</p>}
    </div>
  );
}
```

### List Rendering

```tsx
function TodoList() {
  const todos = signal([
    { id: 1, text: "Learn Flick" },
    { id: 2, text: "Build something" },
  ]);

  return (
    <ul>
      {todos().map((todo) => (
        <li key={todo.id}>{todo.text}</li>
      ))}
    </ul>
  );
}
```

---

## Event Handling

Use lowercase event names (standard DOM events):

```tsx
function Form() {
  const value = signal("");

  return (
    <div>
      <input
        type="text"
        value={value()}
        oninput={(e) => value.set(e.target.value)}
      />
      <p>You typed: {value()}</p>

      <button onclick={() => alert("Clicked!")}>Click me</button>

      <div
        onmouseenter={() => console.log("Mouse entered")}
        onmouseleave={() => console.log("Mouse left")}
      >
        Hover over me
      </div>
    </div>
  );
}
```

---

## Styling

### Class Attribute

Use `class` (not `className`):

```tsx
function StyledComponent() {
  const isActive = signal(false);

  return (
    <div class={isActive() ? "active" : "inactive"}>
      <button class="btn btn-primary" onclick={() => isActive.set(!isActive())}>
        Toggle
      </button>
    </div>
  );
}
```

### Inline Styles

```tsx
function ColorBox() {
  const color = signal("red");

  return (
    <div style={`background-color: ${color()}; padding: 20px;`}>
      <button onclick={() => color.set("blue")}>Make Blue</button>
      <button onclick={() => color.set("red")}>Make Red</button>
    </div>
  );
}
```

---

## Tailwind CSS Setup


### Installation

```bash
bun add -D tailwindcss @tailwindcss/vite
```

### Configuration

**1. Create `src/index.css`:**

```css
@import "tailwindcss";
```

**2. Update `vite.config.js`:**

```js
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
// ... your existing flickPlugin

export default defineConfig({
  plugins: [
    flickPlugin(),
    tailwindcss(),
  ],
});
```

**3. Import CSS in `src/main.tsx`:**

```tsx
import "./index.css";
import { signal, mount } from "@flickjs/runtime";

// ... rest of your app
```

### Usage Example

```tsx
import "./index.css";
import { signal, mount } from "@flickjs/runtime";

function Counter() {
  const count = signal(0);

  return (
    <div class="min-h-screen bg-gray-100 flex items-center justify-center">
      <div class="bg-white p-8 rounded-lg shadow-lg">
        <h1 class="text-3xl font-bold text-gray-800 mb-4">
          Count: {count()}
        </h1>
        <div class="flex gap-2">
          <button
            class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
            onclick={() => count.set(count() + 1)}
          >
            Increment
          </button>
          <button
            class="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded"
            onclick={() => count.set(count() - 1)}
          >
            Decrement
          </button>
        </div>
      </div>
    </div>
  );
}

mount(Counter, document.getElementById("app"));
```

---

## File-Based Router (Optional)

For multi-page apps, use the Flick router:

### Installation

```bash
bun add @flickjs/router
```

### Setup

Update `vite.config.js`:

```js
import { defineConfig } from "vite";
import { flickRouter } from "@flickjs/router/vite";
// ... your flickPlugin from above

export default defineConfig({
  plugins: [flickPlugin(), flickRouter({ pagesDir: "pages" })],
});
```

### Directory Structure

```
src/
├── pages/
│   ├── index.tsx      → /
│   ├── about.tsx      → /about
│   ├── blog/
│   │   ├── index.tsx  → /blog
│   │   └── [slug].tsx → /blog/:slug (dynamic route)
│   └── [...all].tsx   → /* (catch-all/404)
└── main.tsx
```

### Main Entry Point

```tsx
// src/main.tsx
import { mount } from "@flickjs/runtime";
import { Router, Link } from "@flickjs/router";
import { routes } from "virtual:flick-routes";

function App() {
  return (
    <div>
      <nav>
        <Link href="/">Home</Link>
        <Link href="/about">About</Link>
        <Link href="/blog">Blog</Link>
      </nav>

      <main>
        <Router routes={routes} />
      </main>
    </div>
  );
}

mount(App, document.getElementById("app"));
```

### Page Components

```tsx
// src/pages/index.tsx
export default function Home() {
  return <h1>Welcome Home!</h1>;
}

// src/pages/about.tsx
export default function About() {
  return <h1>About Us</h1>;
}

// src/pages/blog/[slug].tsx
import { useParams } from "@flickjs/router";

export default function BlogPost() {
  const params = useParams();
  return <h1>Blog Post: {params().slug}</h1>;
}
```

### Navigation

```tsx
import { Link, navigate } from "@flickjs/router";

function Navigation() {
  return (
    <nav>
      {/* Declarative navigation */}
      <Link href="/">Home</Link>
      <Link href="/about">About</Link>

      {/* Programmatic navigation */}
      <button onclick={() => navigate("/contact")}>Go to Contact</button>
    </nav>
  );
}
```

### Get Current Route

```tsx
import { useRoute } from "@flickjs/router";

function Breadcrumb() {
  const route = useRoute();

  return <p>Current path: {route().path}</p>;
}
```

---

## TypeScript Support

Flick works with TypeScript out of the box. Add a `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*"]
}
```

### Typing Signals

```tsx
import { signal } from "@flickjs/runtime";

// Type is inferred
const count = signal(0); // Signal<number>

// Explicit typing
const user = signal<{ name: string; age: number } | null>(null);

// Update with type safety
user.set({ name: "John", age: 30 });
```

---

## Building for Production

```bash
# Build
bun run build

# Preview production build
bun run preview
```

The built files will be in the `dist/` directory, ready for deployment.

---

## Project Structure Example

```
my-flick-app/
├── src/
│   ├── components/
│   │   ├── Button.tsx
│   │   └── Card.tsx
│   ├── pages/           # If using router
│   │   ├── index.tsx
│   │   └── about.tsx
│   └── main.tsx
├── index.html
├── vite.config.js
├── tsconfig.json
└── package.json
```

---

## Tips & Best Practices

1. **Keep signals at the top level** - Define signals at the beginning of your component
2. **Use derived values for computed state** - Instead of effects that update signals
3. **Fine-grained updates** - Flick only updates the specific DOM nodes that depend on changed signals
4. **No virtual DOM** - Direct DOM manipulation for maximum performance
5. **Standard HTML attributes** - Use `class` and not `className`, `onclick` and not `onClick`

---

---

## Resources

- [GitHub Repository](https://github.com/jaymalve/flickjs)
- [Report Issues](https://github.com/jaymalve/flickjs/issues)

---
