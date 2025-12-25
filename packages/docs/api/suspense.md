# Suspense & Async Data

Flick provides built-in support for handling asynchronous operations with `Suspense`, `query`, and `lazy`.

## Suspense

The `Suspense` component displays a fallback UI while async operations are pending.

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

### Key Points

- `fallback` is displayed while any async operations inside are pending
- Once all queries resolve, the children are shown
- Suspense boundaries can be nested for granular loading states

---

## Query

`query` creates an async data fetcher that integrates with Suspense.

### Simple Query

```tsx
import { query, Suspense } from "@flickjs/runtime";

const posts = query(() => fetch("/api/posts").then((res) => res.json()));

function PostList() {
  return (
    <ul>
      {posts()?.map((post) => (
        <li>{post.title}</li>
      ))}
    </ul>
  );
}
```

### Query with Reactive Source

When the source fx changes, the query automatically refetches:

```tsx
import { fx, query, Suspense } from "@flickjs/runtime";

function UserPosts() {
  const userId = fx(1);

  const posts = query(
    userId, // Source - refetches when this changes
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

### Query API

| Method           | Returns              | Description                                   |
| ---------------- | -------------------- | --------------------------------------------- |
| `query()`        | `T \| undefined`     | Current value (undefined while loading)       |
| `query.loading()`| `boolean`            | True while fetching                           |
| `query.error()`  | `Error \| undefined` | Error if fetch failed                         |
| `query.latest()` | `T \| undefined`     | Last successful value (useful during refetch) |
| `query.refetch()`| `void`               | Manually trigger a refetch                    |

---

## Lazy Loading

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

---

## Nested Suspense

Use nested Suspense boundaries for granular loading states:

```tsx
function Dashboard() {
  return (
    <div>
      <Suspense fallback={<p>Loading header...</p>}>
        <Header />
      </Suspense>

      <div class="grid">
        <Suspense fallback={<p>Loading stats...</p>}>
          <Stats />
        </Suspense>

        <Suspense fallback={<p>Loading chart...</p>}>
          <Chart />
        </Suspense>
      </div>
    </div>
  );
}
```

---

## Complete Example

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

## Error Handling

Handle errors at the query level:

```tsx
function UserProfile() {
  const user = query(() => fetchUser());

  // Check error state
  if (user.error()) {
    return (
      <div class="error">
        <p>Failed to load user: {user.error()?.message}</p>
        <button onclick={() => user.refetch()}>Retry</button>
      </div>
    );
  }

  if (user.loading()) {
    return <p>Loading...</p>;
  }

  return (
    <div>
      <h1>{user()?.name}</h1>
      <p>{user()?.email}</p>
    </div>
  );
}
```

---

## Best Practices

1. **Place Suspense boundaries strategically** - Too high causes everything to show loading; too low causes many loading states
2. **Use `latest()` for better UX** - Show stale data while refetching new data
3. **Handle errors explicitly** - Check `query.error()` and provide retry functionality
4. **Lazy load heavy components** - Use `lazy()` for components not needed on initial load
