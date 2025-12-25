# Runtime API

The `@flickjs/runtime` package provides the core reactive primitives.

```bash
bun add @flickjs/runtime
```

## fx

Creates a reactive fx.

```tsx
import { fx } from "@flickjs/runtime";

const count = fx(0);

// Read value
console.log(count()); // 0

// Set value
count.set(5);
console.log(count()); // 5
```

### Type Signature

```ts
function fx<T>(initialValue: T): Fx<T>;

interface Fx<T> {
  (): T; // Read the value
  set(value: T): void; // Set a new value
}
```

### Parameters

| Parameter      | Type | Description                  |
| -------------- | ---- | ---------------------------- |
| `initialValue` | `T`  | The initial value of the fx  |

### Returns

A fx object that can be called to read the value and has a `set` method to update it.

---

## run

Runs a side effect when its dependencies change.

```tsx
import { fx, run } from "@flickjs/runtime";

const count = fx(0);

run(() => {
  console.log("Count is:", count());
});

count.set(1); // Logs: "Count is: 1"
```

### Type Signature

```ts
function run(fn: () => void): void;
```

### Parameters

| Parameter | Type         | Description           |
| --------- | ------------ | --------------------- |
| `fn`      | `() => void` | The run function      |

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

| Parameter   | Type              | Description                     |
| ----------- | ----------------- | ------------------------------- |
| `component` | `() => Element`   | The component function to mount |
| `container` | `Element \| null` | The DOM element to mount into   |

---

## Suspense

A component that shows fallback content while async operations are pending.

```tsx
import { Suspense, query } from "@flickjs/runtime";

function UserProfile() {
  const user = query(() => fetchUser());
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

| Prop       | Type      | Description                   |
| ---------- | --------- | ----------------------------- |
| `fallback` | `Element` | Content to show while loading |
| `children` | `Element` | Content to show when loaded   |

---

## query

Creates an async data fetcher that integrates with Suspense.

```tsx
import { fx, query } from "@flickjs/runtime";

// Simple query
const posts = query(() => fetch("/api/posts").then((res) => res.json()));

// Query with reactive source
const userId = fx(1);
const user = query(userId, (id) =>
  fetch(`/api/users/${id}`).then((r) => r.json())
);
```

### Type Signature

```ts
// Without source
function query<T>(fetcher: () => Promise<T>): Query<T>;

// With source
function query<T, S>(
  source: () => S,
  fetcher: (source: S) => Promise<T>
): Query<T>;

interface Query<T> {
  (): T | undefined; // Current value
  loading(): boolean; // Is fetching
  error(): Error | undefined; // Error if failed
  latest(): T | undefined; // Last successful value
  refetch(): void; // Manually refetch
}
```

### Query Properties

| Property    | Type                 | Description                   |
| ----------- | -------------------- | ----------------------------- |
| `()`        | `T \| undefined`     | Returns current value         |
| `loading()` | `boolean`            | Returns true while fetching   |
| `error()`   | `Error \| undefined` | Returns error if fetch failed |
| `latest()`  | `T \| undefined`     | Returns last successful value |
| `refetch()` | `void`               | Manually triggers a refetch   |

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
function lazy<T>(loader: () => Promise<{ default: T }>): () => T;
```

### Parameters

| Parameter | Type                            | Description             |
| --------- | ------------------------------- | ----------------------- |
| `loader`  | `() => Promise<{ default: T }>` | Dynamic import function |
