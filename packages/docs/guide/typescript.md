# TypeScript

Flick works with TypeScript out of the box.

## Configuration

Add a `tsconfig.json`:

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

## Typing Signals

```tsx
import { signal } from "@flickjs/runtime";

// Type is inferred
const count = signal(0); // Signal<number>

// Explicit typing
const user = signal<{ name: string; age: number } | null>(null);

// Update with type safety
user.set({ name: "John", age: 30 });
```

## Typing Components

### Props Interface

```tsx
interface ButtonProps {
  children: any;
  onclick: () => void;
  variant?: "primary" | "secondary" | "danger";
  disabled?: boolean;
}

function Button({
  children,
  onclick,
  variant = "primary",
  disabled = false,
}: ButtonProps) {
  return (
    <button class={`btn btn-${variant}`} onclick={onclick} disabled={disabled}>
      {children}
    </button>
  );
}
```

### Generic Components

```tsx
interface ListProps<T> {
  items: T[];
  renderItem: (item: T) => any;
}

function List<T>({ items, renderItem }: ListProps<T>) {
  return (
    <ul>
      {items.map((item) => (
        <li>{renderItem(item)}</li>
      ))}
    </ul>
  );
}

// Usage
<List items={[1, 2, 3]} renderItem={(num) => <span>{num * 2}</span>} />;
```

## Typing Events

```tsx
function Form() {
  const value = signal("");

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    value.set(target.value);
  };

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    console.log("Submitted:", value());
  };

  return (
    <form onsubmit={handleSubmit}>
      <input type="text" value={value()} oninput={handleInput} />
      <button type="submit">Submit</button>
    </form>
  );
}
```

## Type Definitions for Router

When using the router, add type definitions for dynamic params:

```tsx
import { params } from "@flickjs/router";

interface BlogParams {
  slug: string;
}

function BlogPost() {
  const typedParams = params<BlogParams>();

  return <h1>Blog Post: {typedParams.slug}</h1>;
}
```

## Type Definitions for Resources

```tsx
import { resource } from "@flickjs/runtime";

interface User {
  id: number;
  name: string;
  email: string;
}

const user = resource<User>(() => fetch("/api/user").then((res) => res.json()));

// user() returns User | undefined
// user.loading() returns boolean
// user.error() returns Error | undefined
```

## Strict Null Checks

With strict mode enabled, handle potential undefined values:

```tsx
const user = resource<User>(() => fetchUser());

function Profile() {
  // Handle loading and error states
  if (user.loading()) return <p>Loading...</p>;
  if (user.error()) return <p>Error: {user.error()?.message}</p>;

  // Now TypeScript knows user() is defined
  const userData = user()!;

  return (
    <div>
      <h1>{userData.name}</h1>
      <p>{userData.email}</p>
    </div>
  );
}
```
