# Effects

Effects run side effects when signals change. They automatically track which signals they depend on and re-run when those signals update.

## Basic Usage

```tsx
import { signal, effect, mount } from "@flickjs/runtime";

function Logger() {
  const count = signal(0);

  // Runs whenever count changes
  effect(() => {
    console.log("Count changed to:", count());
  });

  return (
    <button onclick={() => count.set(count() + 1)}>
      Click me ({count()})
    </button>
  );
}
```

## Use Cases

### Logging and Debugging

```tsx
function Debug() {
  const data = signal(null);

  effect(() => {
    console.log("Data updated:", data());
  });

  // ...
}
```

### Syncing with External Systems

```tsx
function LocalStorageSync() {
  const theme = signal(localStorage.getItem("theme") || "light");

  effect(() => {
    localStorage.setItem("theme", theme());
  });

  return (
    <button onclick={() => theme.set(theme() === "light" ? "dark" : "light")}>
      Toggle Theme ({theme()})
    </button>
  );
}
```

### Document Title

```tsx
function PageTitle() {
  const title = signal("Home");

  effect(() => {
    document.title = title();
  });

  return (
    <div>
      <input
        value={title()}
        oninput={(e) => title.set(e.target.value)}
      />
    </div>
  );
}
```

### External Library Integration

```tsx
function ChartComponent() {
  const data = signal([1, 2, 3, 4, 5]);
  let chartRef;

  effect(() => {
    if (chartRef) {
      // Update external chart library when data changes
      updateChart(chartRef, data());
    }
  });

  return <canvas ref={(el) => (chartRef = el)} />;
}
```

## Best Practices

### 1. Prefer Derived Values

For computed state, use derived values instead of effects that update signals:

```tsx
// ❌ Don't do this
function Bad() {
  const price = signal(100);
  const quantity = signal(2);
  const total = signal(0);

  effect(() => {
    total.set(price() * quantity());
  });
}

// ✅ Do this
function Good() {
  const price = signal(100);
  const quantity = signal(2);
  const total = () => price() * quantity(); // Derived value
}
```

### 2. Keep Effects Simple

Effects should handle one specific side effect:

```tsx
// ❌ Too much in one effect
effect(() => {
  localStorage.setItem("user", user());
  document.title = user().name;
  analytics.track("user_update", user());
});

// ✅ Separate concerns
effect(() => localStorage.setItem("user", user()));
effect(() => document.title = user().name);
effect(() => analytics.track("user_update", user()));
```

### 3. Avoid Circular Dependencies

Don't create effects that update signals they read from:

```tsx
// ❌ This creates an infinite loop
const count = signal(0);
effect(() => {
  count.set(count() + 1); // Never do this!
});
```
