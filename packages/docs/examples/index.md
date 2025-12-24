# Examples

## Project Structure

A typical Flick application has the following structure:

```
my-flick-app/
├── src/
│   ├── components/        # Reusable components
│   │   ├── Button.tsx
│   │   └── Card.tsx
│   ├── pages/             # File-based routing (if using router)
│   │   ├── index.tsx      # / route
│   │   ├── about.tsx      # /about route
│   │   └── blog/
│   │       ├── index.tsx  # /blog route
│   │       └── [slug].tsx # /blog/:slug route
│   ├── stores/            # Shared state
│   │   └── user.store.ts
│   ├── types/             # TypeScript types
│   │   └── user.ts
│   ├── index.css          # Global styles
│   └── main.tsx           # Application entry point
├── public/                # Static assets
├── index.html             # HTML entry point
├── vite.config.js         # Vite configuration
├── tsconfig.json          # TypeScript configuration
└── package.json
```

---

## Counter Example

A simple counter demonstrating signals:

```tsx
import { signal, mount } from "@flickjs/runtime";

function Counter() {
  const count = signal(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={() => count.set(count() + 1)}>+</button>
      <button onclick={() => count.set(count() - 1)}>-</button>
      <button onclick={() => count.set(0)}>Reset</button>
    </div>
  );
}

mount(Counter, document.getElementById("app"));
```

---

## Todo List Example

A todo list demonstrating state management and list rendering:

```tsx
import { signal, mount } from "@flickjs/runtime";

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

function TodoApp() {
  const todos = signal<Todo[]>([]);
  const input = signal("");
  let nextId = 1;

  const addTodo = () => {
    if (input().trim()) {
      todos.set([...todos(), { id: nextId++, text: input(), done: false }]);
      input.set("");
    }
  };

  const toggleTodo = (id: number) => {
    todos.set(
      todos().map((todo) =>
        todo.id === id ? { ...todo, done: !todo.done } : todo
      )
    );
  };

  const removeTodo = (id: number) => {
    todos.set(todos().filter((todo) => todo.id !== id));
  };

  return (
    <div>
      <h1>Todo List</h1>

      <div>
        <input
          type="text"
          value={input()}
          oninput={(e) => input.set(e.target.value)}
          onkeypress={(e) => e.key === "Enter" && addTodo()}
          placeholder="Add a todo..."
        />
        <button onclick={addTodo}>Add</button>
      </div>

      <ul>
        {todos().map((todo) => (
          <li key={todo.id}>
            <input
              type="checkbox"
              checked={todo.done}
              onchange={() => toggleTodo(todo.id)}
            />
            <span style={todo.done ? "text-decoration: line-through" : ""}>
              {todo.text}
            </span>
            <button onclick={() => removeTodo(todo.id)}>Delete</button>
          </li>
        ))}
      </ul>

      <p>
        {todos().filter((t) => !t.done).length} items remaining
      </p>
    </div>
  );
}

mount(TodoApp, document.getElementById("app"));
```

---

## Data Fetching Example

Fetching data with Suspense and resources:

```tsx
import { signal, mount, Suspense, resource } from "@flickjs/runtime";

interface User {
  id: number;
  name: string;
  email: string;
}

function UserList() {
  const users = resource<User[]>(() =>
    fetch("https://jsonplaceholder.typicode.com/users").then((r) => r.json())
  );

  return (
    <div>
      <h2>Users</h2>
      {users.error() && <p>Error: {users.error()?.message}</p>}
      <ul>
        {users()?.map((user) => (
          <li key={user.id}>
            <strong>{user.name}</strong> - {user.email}
          </li>
        ))}
      </ul>
    </div>
  );
}

function App() {
  return (
    <div>
      <h1>User Directory</h1>
      <Suspense fallback={<p>Loading users...</p>}>
        <UserList />
      </Suspense>
    </div>
  );
}

mount(App, document.getElementById("app"));
```

---

## Theme Toggle Example

Implementing a dark mode toggle with effects:

```tsx
import { signal, effect, mount } from "@flickjs/runtime";

function ThemeToggle() {
  const theme = signal(localStorage.getItem("theme") || "light");

  // Sync to localStorage and document
  effect(() => {
    localStorage.setItem("theme", theme());
    document.documentElement.setAttribute("data-theme", theme());
  });

  return (
    <button onclick={() => theme.set(theme() === "light" ? "dark" : "light")}>
      {theme() === "light" ? "🌙 Dark Mode" : "☀️ Light Mode"}
    </button>
  );
}

mount(ThemeToggle, document.getElementById("app"));
```

---

## Routing Example

Using file-based routing:

```tsx
// src/main.tsx
import "./index.css";
import { mount, Suspense } from "@flickjs/runtime";
import { Router, Link } from "@flickjs/router";
import { routes } from "virtual:flick-routes";

function App() {
  return (
    <div>
      <nav>
        <Link href="/">Home</Link>
        <Link href="/about">About</Link>
        <Link href="/users">Users</Link>
      </nav>

      <main>
        <Suspense fallback={<p>Loading page...</p>}>
          <Router routes={routes} />
        </Suspense>
      </main>
    </div>
  );
}

mount(App, document.getElementById("app"));
```

```tsx
// src/pages/index.tsx
export default function Home() {
  return (
    <div>
      <h1>Welcome to Flick</h1>
      <p>A tiny reactive JS framework from the future.</p>
    </div>
  );
}
```

```tsx
// src/pages/users/[id].tsx
import { useParams } from "@flickjs/router";
import { resource, Suspense } from "@flickjs/runtime";

export default function UserPage() {
  const params = useParams();

  const user = resource(
    () => params().id,
    (id) => fetch(`/api/users/${id}`).then((r) => r.json())
  );

  return (
    <div>
      <h1>User Profile</h1>
      {user.loading() && <p>Loading...</p>}
      {user.error() && <p>Error loading user</p>}
      {user() && (
        <div>
          <h2>{user()?.name}</h2>
          <p>{user()?.email}</p>
        </div>
      )}
    </div>
  );
}
```

---

## Tips & Best Practices

1. **Keep signals at the top level** - Define signals at the beginning of your component
2. **Use derived values for computed state** - Instead of effects that update signals
3. **Fine-grained updates** - Flick only updates the specific DOM nodes that depend on changed signals
4. **No virtual DOM** - Direct DOM manipulation for maximum performance
5. **Standard HTML attributes** - Use `class` not `className`, `onclick` not `onClick`
