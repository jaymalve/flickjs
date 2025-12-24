# Runtime API

The `@flickjs/runtime` package provides the core reactive primitives.

```bash
bun add @flickjs/runtime
```

## signal

Creates a reactive signal.

```tsx
import { signal } from "@flickjs/runtime";

const count = signal(0);

// Read value
console.log(count()); // 0

// Set value
count.set(5);
console.log(count()); // 5
```

### Type Signature

```ts
function signal<T>(initialValue: T): Signal<T>;

interface Signal<T> {
  (): T;                    // Read the value
  set(value: T): void;      // Set a new value
}
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `initialValue` | `T` | The initial value of the signal |

### Returns

A signal object that can be called to read the value and has a `set` method to update it.

---

## effect

Runs a side effect when its dependencies change.

```tsx
import { signal, effect } from "@flickjs/runtime";

const count = signal(0);

effect(() => {
  console.log("Count is:", count());
});

count.set(1); // Logs: "Count is: 1"
```

### Type Signature

```ts
function effect(fn: () => void): void;
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `fn` | `() => void` | The effect function to run |

---

## mount

Mounts a component to a DOM element.

```tsx
import { mount } from "@flickjs/runtime";

function App() {
  return <h1>Hello World</h1>;
}

mount(App, document.getElementById("app"));
```

### Type Signature

```ts
function mount(component: () => Element, container: Element | null): void;
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `component` | `() => Element` | The component function to mount |
| `container` | `Element \| null` | The DOM element to mount into |

---

## Suspense

A component that shows fallback content while async operations are pending.

```tsx
import { Suspense, resource } from "@flickjs/runtime";

function UserProfile() {
  const user = resource(() => fetchUser());
  return <div>{user()?.name}</div>;
}

function App() {
  return (
    <Suspense fallback={<p>Loading...</p>}>
      <UserProfile />
    </Suspense>
  );
}
```

### Props

| Prop | Type | Description |
|------|------|-------------|
| `fallback` | `Element` | Content to show while loading |
| `children` | `Element` | Content to show when loaded |

---

## resource

Creates an async data fetcher that integrates with Suspense.

```tsx
import { signal, resource } from "@flickjs/runtime";

// Simple resource
const posts = resource(() =>
  fetch("/api/posts").then((res) => res.json())
);

// Resource with reactive source
const userId = signal(1);
const user = resource(
  () => userId(),
  (id) => fetch(`/api/users/${id}`).then((r) => r.json())
);
```

### Type Signature

```ts
// Without source
function resource<T>(fetcher: () => Promise<T>): Resource<T>;

// With source
function resource<T, S>(
  source: () => S,
  fetcher: (source: S) => Promise<T>
): Resource<T>;

interface Resource<T> {
  (): T | undefined;           // Current value
  loading(): boolean;          // Is fetching
  error(): Error | undefined;  // Error if failed
  latest(): T | undefined;     // Last successful value
  refetch(): void;             // Manually refetch
}
```

### Resource Properties

| Property | Type | Description |
|----------|------|-------------|
| `()` | `T \| undefined` | Returns current value |
| `loading()` | `boolean` | Returns true while fetching |
| `error()` | `Error \| undefined` | Returns error if fetch failed |
| `latest()` | `T \| undefined` | Returns last successful value |
| `refetch()` | `void` | Manually triggers a refetch |

---

## lazy

Lazily loads a component for code splitting.

```tsx
import { lazy, Suspense } from "@flickjs/runtime";

const HeavyComponent = lazy(() => import("./HeavyComponent"));

function App() {
  return (
    <Suspense fallback={<p>Loading...</p>}>
      <HeavyComponent />
    </Suspense>
  );
}
```

### Type Signature

```ts
function lazy<T>(
  loader: () => Promise<{ default: T }>
): () => T;
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `loader` | `() => Promise<{ default: T }>` | Dynamic import function |
