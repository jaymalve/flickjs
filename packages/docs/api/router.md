# Router API

The `@flickjs/router` package provides file-based routing for Flick applications.

```bash
bun add @flickjs/router
```

## Setup

### Vite Configuration

```js
import { defineConfig } from "vite";
import { flickRouter } from "@flickjs/router/vite";

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

---

## Router

The main router component that renders the current route.

```tsx
import { Router } from "@flickjs/router";
import { routes } from "virtual:flick-routes";

function App() {
  return (
    <div>
      <nav>{/* navigation */}</nav>
      <main>
        <Router routes={routes} />
      </main>
    </div>
  );
}
```

### Props

| Prop     | Type      | Description                |
| -------- | --------- | -------------------------- |
| `routes` | `Route[]` | Array of route definitions |

---

## Link

A navigation link component for client-side routing.

```tsx
import { Link } from "@flickjs/router";

function Navigation() {
  return (
    <nav>
      <Link href="/">Home</Link>
      <Link href="/about">About</Link>
      <Link href="/blog">Blog</Link>
    </nav>
  );
}
```

### Props

| Prop       | Type      | Description             |
| ---------- | --------- | ----------------------- |
| `href`     | `string`  | The path to navigate to |
| `children` | `Element` | Link content            |

---

## navigate

Programmatically navigate to a route.

```tsx
import { navigate } from "@flickjs/router";

function LoginForm() {
  const handleSubmit = async () => {
    await login();
    navigate("/dashboard");
  };

  return (
    <form onsubmit={handleSubmit}>
      {/* form fields */}
      <button type="submit">Login</button>
    </form>
  );
}
```

### Type Signature

```ts
function navigate(path: string): void;
```

### Parameters

| Parameter | Type     | Description             |
| --------- | -------- | ----------------------- |
| `path`    | `string` | The path to navigate to |

---

## currentPath

Get the current route information.

```tsx
import { currentPath } from "@flickjs/router";

function Breadcrumb() {
  const route = currentPath();

  return <p>Current path: {route}</p>;
}
```

### Returns

A function that returns the current route object with `path` and `params`.

---

## params

Get the current route parameters.

```tsx
import { params } from "@flickjs/router";

// For route /blog/[slug].tsx
function BlogPost() {

  return <h1>Blog Post: {params().slug}</h1>;
}
```


### Returns

A function that returns an object with the current route parameters.

---

## Route Patterns

### Static Routes

```
pages/index.tsx     → /
pages/about.tsx     → /about
pages/contact.tsx   → /contact
```

### Nested Routes

```
pages/blog/index.tsx    → /blog
pages/blog/archive.tsx  → /blog/archive
```

### Dynamic Routes

Use `[param]` for dynamic segments:

```
pages/users/[id].tsx        → /users/:id
pages/blog/[slug].tsx       → /blog/:slug
pages/[category]/[id].tsx   → /:category/:id
```

### Catch-All Routes

Use `[...param]` for catch-all routes:

```
pages/[...all].tsx          → /* (404 page)
pages/docs/[...path].tsx    → /docs/* (nested docs)
```

---

## Complete Example

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

```tsx
// src/pages/index.tsx
export default function Home() {
  return <h1>Welcome Home!</h1>;
}
```

```tsx
// src/pages/blog/[slug].tsx
import { params } from "@flickjs/router";

export default function BlogPost() {
  return <h1>Blog Post: {params().slug}</h1>;
}
```
