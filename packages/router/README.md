# @flickjs/router

File-based router for Flick framework with History API support, dynamic routes, and reactive routing state.

## Installation

```bash
bun add @flickjs/router
# or
npm install @flickjs/router
```

## Quick Start

### 1. Set Up Vite Plugin

Add the router plugin to your `vite.config.js`:

```javascript
import { defineConfig } from "vite";
import { flickRouter } from "@flickjs/router/vite";

export default defineConfig({
  plugins: [
    flickRouter({ pagesDir: "pages", root: process.cwd() }),
    // ... other plugins
  ],
});
```

### 2. Create Pages Directory

Create a `src/pages/` directory and add your route files:

```
src/
└── pages/
    ├── index.tsx      # Route: /
    ├── about.tsx      # Route: /about
    └── users/
        └── [id].tsx   # Route: /users/:id
```

### 3. Use Router in Your App

Update your `src/main.tsx`:

```tsx
import { mount } from "@flickjs/runtime";
import { Router } from "@flickjs/router";
import routes from "virtual:flick-routes";

mount(() => <Router routes={routes} />, document.getElementById("app"));
```

### 4. Create Your First Page

Create `src/pages/index.tsx`:

```tsx
import { Link } from "@flickjs/router";

export default function Home() {
  return (
    <div>
      <h1>Home Page</h1>
      <Link href="/about">Go to About</Link>
    </div>
  );
}
```

## File-Based Routing

Routes are automatically generated from your file structure. The router uses a convention-based approach similar to Next.js and SvelteKit.

### Route Conventions

| File Path                   | Route          | Description             |
| --------------------------- | -------------- | ----------------------- |
| `pages/index.tsx`           | `/`            | Home page               |
| `pages/about.tsx`           | `/about`       | Static route            |
| `pages/users/index.tsx`     | `/users`       | Nested index            |
| `pages/users/[id].tsx`      | `/users/:id`   | Dynamic route parameter |
| `pages/posts/[...slug].tsx` | `/posts/*slug` | Catch-all route         |

### Dynamic Routes

Use square brackets `[param]` for dynamic route parameters:

**File**: `pages/users/[id].tsx`

```tsx
import { params } from "@flickjs/router";

export default function UserPage() {
  return <h1>User ID: {params().id}</h1>;
}
```

Accessing `/users/123` will display "User ID: 123".

### Catch-All Routes

Use `[...param]` for catch-all routes that match multiple segments:

**File**: `pages/posts/[...slug].tsx`

```tsx
import { params } from "@flickjs/router";

export default function PostPage() {
  return <h1>Post: {params().slug}</h1>;
}
```

Accessing `/posts/hello/world` will display "Post: hello/world".

## API

### `Router`

Main router component that handles route matching and rendering.

**Props**:

- `routes`: Array of route objects (auto-generated from `virtual:flick-routes`)

**Example**:

```tsx
import { Router } from "@flickjs/router";
import routes from "virtual:flick-routes";

<Router routes={routes} />;
```

### `Link`

Component for declarative navigation. Intercepts clicks and uses client-side navigation.

**Props**:

- `href`: Destination URL
- `children`: Link content
- Other HTML anchor attributes (`class`, `id`, etc.)

**Example**:

```tsx
import { Link } from "@flickjs/router";

<Link href="/about" class="nav-link">
  About
</Link>;
```

### `navigate(to, options?)`

Programmatic navigation function.

**Parameters**:

- `to`: Destination URL string
- `options`: Optional configuration
  - `replace`: If `true`, replaces current history entry instead of adding new one

**Example**:

```tsx
import { navigate } from "@flickjs/router";

function handleClick() {
  navigate("/about");
}

// Replace instead of push
navigate("/login", { replace: true });
```

### `currentPath()`

Reactive signal that returns the current URL pathname.

**Example**:

```tsx
import { currentPath } from "@flickjs/router";

function NavBar() {
  const path = currentPath();

  return (
    <nav>
      <p>Current path: {path}</p>
    </nav>
  );
}
```

### `params()`

Reactive signal that returns route parameters as an object.

**Example**:

```tsx
import { params } from "@flickjs/router";

export default function UserPage() {
  const routeParams = params();

  return (
    <div>
      <h1>User: {routeParams.id}</h1>
    </div>
  );
}
```

### `query()`

Reactive signal that returns URL query parameters as a `URLSearchParams` object.

**Example**:

```tsx
import { query } from "@flickjs/router";

export default function SearchPage() {
  const searchParams = query();
  const search = searchParams.get("q") || "";

  return <h1>Search: {search}</h1>;
}
```

Accessing `/search?q=hello` will display "Search: hello".

## Complete Examples

### Basic Navigation

```tsx
// src/pages/index.tsx
import { Link } from "@flickjs/router";

export default function Home() {
  return (
    <div>
      <h1>Home</h1>
      <nav>
        <Link href="/about">About</Link>
        <Link href="/contact">Contact</Link>
      </nav>
    </div>
  );
}
```

### Dynamic Route with Parameters

```tsx
// src/pages/users/[id].tsx
import { params, Link } from "@flickjs/router";

export default function UserPage() {
  const userId = params().id;

  return (
    <div>
      <h1>User Profile</h1>
      <p>User ID: {userId}</p>
      <Link href="/">Back to Home</Link>
    </div>
  );
}
```

### Programmatic Navigation

```tsx
import { signal } from "@flickjs/runtime";
import { navigate } from "@flickjs/router";

function LoginForm() {
  const username = signal("");
  const password = signal("");

  const handleSubmit = () => {
    // Login logic...
    navigate("/dashboard");
  };

  return (
    <form
      onsubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}
    >
      <input
        value={username()}
        oninput={(e) => username.set(e.target.value)}
        placeholder="Username"
      />
      <input
        type="password"
        value={password()}
        oninput={(e) => password.set(e.target.value)}
        placeholder="Password"
      />
      <button type="submit">Login</button>
    </form>
  );
}
```

### Query Parameters

```tsx
// src/pages/search.tsx
import { query, navigate } from "@flickjs/router";
import { signal } from "@flickjs/runtime";

export default function SearchPage() {
  const searchParams = query();
  const searchTerm = signal(searchParams.get("q") || "");

  const handleSearch = () => {
    navigate(`/search?q=${encodeURIComponent(searchTerm())}`);
  };

  return (
    <div>
      <input
        value={searchTerm()}
        oninput={(e) => searchTerm.set(e.target.value)}
        placeholder="Search..."
      />
      <button onclick={handleSearch}>Search</button>
      {searchParams.get("q") && <p>Results for: {searchParams.get("q")}</p>}
    </div>
  );
}
```

### Nested Routes

```tsx
// src/pages/blog/index.tsx
import { Link } from "@flickjs/router";

export default function BlogIndex() {
  return (
    <div>
      <h1>Blog</h1>
      <ul>
        <li>
          <Link href="/blog/post-1">Post 1</Link>
        </li>
        <li>
          <Link href="/blog/post-2">Post 2</Link>
        </li>
      </ul>
    </div>
  );
}

// src/pages/blog/[slug].tsx
import { params, Link } from "@flickjs/router";

export default function BlogPost() {
  const slug = params().slug;

  return (
    <div>
      <h1>Post: {slug}</h1>
      <Link href="/blog">Back to Blog</Link>
    </div>
  );
}
```

## Advanced Patterns

### Active Link Styling

```tsx
import { Link, currentPath } from "@flickjs/router";

function NavLink({ href, children }) {
  const path = currentPath();
  const isActive = path === href;

  return (
    <Link href={href} class={isActive ? "nav-link active" : "nav-link"}>
      {children}
    </Link>
  );
}
```

### Route Guards

```tsx
import { effect } from "@flickjs/runtime";
import { navigate, currentPath } from "@flickjs/router";
import { signal } from "@flickjs/runtime";

const isAuthenticated = signal(false);

// Protect routes
effect(() => {
  const path = currentPath();
  if (path.startsWith("/dashboard") && !isAuthenticated()) {
    navigate("/login");
  }
});
```

### Loading States

The router automatically handles loading states during route transitions. You can customize the loading UI by modifying the Router component or adding a global loading indicator.

### 404 Page

Create a catch-all route at the end of your routes array or create a `pages/[...all].tsx` file:

```tsx
// src/pages/[...all].tsx
import { currentPath } from "@flickjs/router";

export default function NotFound() {
  const path = currentPath();

  return (
    <div>
      <h1>404 - Page Not Found</h1>
      <p>The page {path} does not exist.</p>
    </div>
  );
}
```

### Route Preloading

Routes are automatically code-split and lazy-loaded. Components are loaded when the route is accessed.

## Vite Plugin Configuration

### Options

The `flickRouter` plugin accepts the following options:

```typescript
flickRouter({
  pagesDir?: string;  // Directory name for pages (default: "pages")
  root?: string;      // Project root directory (default: process.cwd())
})
```

### Example Configuration

```javascript
import { defineConfig } from "vite";
import { flickRouter } from "@flickjs/router/vite";

export default defineConfig({
  plugins: [
    flickRouter({
      pagesDir: "src/pages", // Custom pages directory
      root: __dirname, // Custom root
    }),
  ],
});
```

## Route Matching

Routes are matched in the following order:

1. **Exact matches** - Static routes like `/about`
2. **Dynamic routes** - Routes with parameters like `/users/:id`
3. **Catch-all routes** - Routes with catch-all like `/posts/*slug`

This ensures that more specific routes are matched before more general ones.

## Browser Support

The router uses the History API and works in all modern browsers:

- Chrome 5+
- Firefox 4+
- Safari 5+
- Edge 12+

## Performance

- **Code splitting**: Routes are automatically code-split and lazy-loaded
- **Minimal bundle**: Router adds ~2KB to your bundle
- **Fast matching**: Route matching uses efficient regex patterns
- **Reactive updates**: Only re-renders when route actually changes

## TypeScript Support

The router is fully typed. Import types for better IDE support:

```tsx
import type { Route, MatchResult } from "@flickjs/router";
```

## Troubleshooting

### Routes not found

Make sure:

1. Your `pages/` directory exists in `src/`
2. Files have `.tsx` or `.jsx` extensions
3. The Vite plugin is configured correctly

### Navigation not working

Check that:

1. Links use the `Link` component or `navigate()` function
2. URLs are same-origin (client-side routing only works for same origin)
3. The router is mounted correctly

### Params not updating

Ensure you're reading params reactively:

```tsx
// ✅ Correct
const id = params().id;

// ❌ Wrong - not reactive
const { id } = params();
```

## Migration from Other Routers

### From React Router

**React Router**:

```tsx
<BrowserRouter>
  <Routes>
    <Route path="/about" element={<About />} />
  </Routes>
</BrowserRouter>
```

**Flick Router**:

```tsx
// Just create pages/about.tsx
export default function About() {
  return <div>About</div>;
}
```

### From Next.js

The file structure is very similar. Main differences:

- Use `params()` signal instead of `useParams()`
- Use `Link` from `@flickjs/router` instead of `next/link`
- No `getServerSideProps` or `getStaticProps` - use client-side data fetching

## Links

- [Main Documentation](../../README.md)
- [Runtime Documentation](../runtime/README.md)
- [Compiler Documentation](../compiler/README.md)
- [CLI Documentation](../create-app/README.md)
- [GitHub Repository](https://github.com/jaymalave/flick)
- [npm Package](https://www.npmjs.com/package/@flickjs/router)

## License

MIT © Jay Malave
