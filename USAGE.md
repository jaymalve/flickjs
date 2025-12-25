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

# Install vite plugin (required for JSX)
bun add -D @flickjs/vite-plugin

# Install router (optional)
bun add @flickjs/router
```

### Vite Configuration

Create or update `vite.config.js`:

```js
import { defineConfig } from "vite";
import flick from "@flickjs/vite-plugin";

export default defineConfig({
  plugins: [flick()],
});
```

---

## Core Concepts

### Fx

Fx are reactive values that automatically update the UI when changed.

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

**Key points:**

- `fx(initialValue)` creates a reactive fx
- `count()` reads the current value
- `count.set(newValue)` updates the value and triggers UI updates
- Only the specific DOM nodes that use the fx will update (fine-grained reactivity)

### Derived Values

Computed values that automatically update when their dependencies change:

```tsx
function PriceCalculator() {
  const price = fx(100);
  const quantity = fx(1);

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

### Run

Run side effects when fx change:

```tsx
import { fx, run, mount } from "@flickjs/runtime";

function Logger() {
  const count = fx(0);

  // Runs whenever count changes
  run(() => {
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
  const visible = fx(true);

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
  const todos = fx([
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
  const value = fx("");

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
  const isActive = fx(false);

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
  const color = fx("red");

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
import flick from "@flickjs/vite-plugin";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [flick(), tailwindcss()],
});
```

**3. Import CSS in `src/main.tsx`:**

```tsx
import "./index.css";
import { fx, mount } from "@flickjs/runtime";

// ... rest of your app
```

### Usage Example

```tsx
import "./index.css";
import { fx, mount } from "@flickjs/runtime";

function Counter() {
  const count = fx(0);

  return (
    <div class="min-h-screen bg-gray-100 flex items-center justify-center">
      <div class="bg-white p-8 rounded-lg shadow-lg">
        <h1 class="text-3xl font-bold text-gray-800 mb-4">Count: {count()}</h1>
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

## Suspense & Async Data

Flick provides built-in support for handling asynchronous operations with `Suspense`, `query`, and `lazy` loading.

### Suspense Component

The `Suspense` component displays a fallback UI while async operations are pending:

```tsx
import { mount, Suspense, query } from "@flickjs/runtime";

function UserProfile() {
  const user = query(() => fetch("/api/user").then((res) => res.json()));

  return (
    <div>
      <h1>{user()?.name}</h1>
      <p>{user()?.email}</p>
    </div>
  );
}

function App() {
  return (
    <Suspense fallback={<p>Loading user...</p>}>
      <UserProfile />
    </Suspense>
  );
}

mount(App, document.getElementById("app"));
```

**Key points:**

- `fallback` is displayed while any async operations inside are pending
- Once all queries resolve, the children are shown
- Suspense boundaries can be nested for granular loading states

### Query

`query` creates an async data fetcher that integrates with Suspense:

```tsx
import { fx, query, Suspense } from "@flickjs/runtime";

// Simple query (no source)
const posts = query(() => fetch("/api/posts").then((res) => res.json()));

// Query with reactive source
function UserPosts() {
  const userId = fx(1);

  const posts = query(
    () => userId(), // Source - refetches when this changes
    (id) => fetch(`/api/users/${id}/posts`).then((res) => res.json())
  );

  return (
    <div>
      <button onclick={() => userId.set(userId() + 1)}>Next User</button>

      {posts.loading() && <p>Loading...</p>}
      {posts.error() && <p>Error: {posts.error()?.message}</p>}

      <ul>
        {posts()?.map((post) => (
          <li>{post.title}</li>
        ))}
      </ul>
    </div>
  );
}
```

**Query API:**

- `query()` - returns the current value (or `undefined` while loading)
- `query.loading()` - returns `true` while fetching
- `query.error()` - returns the error if the fetch failed
- `query.latest()` - returns the last successful value (useful during refetch)
- `query.refetch()` - manually trigger a refetch

### Lazy Loading Components

Use `lazy` for code splitting - components are loaded only when needed:

```tsx
import { mount, Suspense, lazy } from "@flickjs/runtime";

// Component is loaded only when rendered
const HeavyChart = lazy(() => import("./components/HeavyChart"));
const Settings = lazy(() => import("./pages/Settings"));

function App() {
  const showChart = fx(false);

  return (
    <div>
      <button onclick={() => showChart.set(!showChart())}>Toggle Chart</button>

      {showChart() && (
        <Suspense fallback={<p>Loading chart...</p>}>
          <HeavyChart data={[1, 2, 3]} />
        </Suspense>
      )}
    </div>
  );
}

mount(App, document.getElementById("app"));
```

### Complete Example

Here's a full example combining Suspense, query, and lazy:

```tsx
import { fx, mount, Suspense, query, lazy } from "@flickjs/runtime";

// Lazy load the chart component
const Chart = lazy(() => import("./Chart"));

function Dashboard() {
  const timeRange = fx("week");

  // Query that refetches when timeRange changes
  const stats = query(
    () => timeRange(),
    (range) => fetch(`/api/stats?range=${range}`).then((r) => r.json())
  );

  return (
    <div>
      <h1>Dashboard</h1>

      <select onchange={(e) => timeRange.set(e.target.value)}>
        <option value="week">This Week</option>
        <option value="month">This Month</option>
        <option value="year">This Year</option>
      </select>

      <Suspense fallback={<p>Loading stats...</p>}>
        <div class="stats">
          <p>Total: {stats()?.total}</p>
          <p>Average: {stats()?.average}</p>
        </div>

        <Suspense fallback={<p>Loading chart...</p>}>
          <Chart data={stats()?.chartData} />
        </Suspense>
      </Suspense>
    </div>
  );
}

mount(Dashboard, document.getElementById("app"));
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
import flick from "@flickjs/vite-plugin";
import { flickRouter } from "@flickjs/router/vite";

export default defineConfig({
  plugins: [flick(), flickRouter({ pagesDir: "pages" })],
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
import { params } from "@flickjs/router";

export default function BlogPost() {
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
import { currentPath } from "@flickjs/router";

function Breadcrumb() {
  const route = currentPath();

  return <p>Current path: {route}</p>;
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

### Typing Fx

```tsx
import { fx } from "@flickjs/runtime";

// Type is inferred
const count = fx(0); // Fx<number>

// Explicit typing
const user = fx<{ name: string; age: number } | null>(null);

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

1. **Keep fx at the top level** - Define fx at the beginning of your component
2. **Use derived values for computed state** - Instead of run effects that update fx
3. **Fine-grained updates** - Flick only updates the specific DOM nodes that depend on changed fx
4. **No virtual DOM** - Direct DOM manipulation for maximum performance
5. **Standard HTML attributes** - Use `class` and not `className`, `onclick` and not `onClick`

---

---

## Resources

- [GitHub Repository](https://github.com/jaymalve/flickjs)
- [Report Issues](https://github.com/jaymalve/flickjs/issues)

---
