# Run

Run executes side effects when fx change. They automatically track which fx they depend on and re-run when those fx update.

## Basic Usage

```tsx
import { fx, run, mount } from "@flickjs/runtime";

function Logger() {
  const count = fx(0);

  // Runs whenever count changes
  run(() => {
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
  const data = fx(null);

  run(() => {
    console.log("Data updated:", data());
  });

  // ...
}
```

### Syncing with External Systems

```tsx
function LocalStorageSync() {
  const theme = fx(localStorage.getItem("theme") || "light");

  run(() => {
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
  const title = fx("Home");

  run(() => {
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
  const data = fx([1, 2, 3, 4, 5]);
  let chartRef;

  run(() => {
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

For computed state, use derived values instead of run that update fx:

```tsx
// Don't do this
function Bad() {
  const price = fx(100);
  const quantity = fx(2);
  const total = fx(0);

  run(() => {
    total.set(price() * quantity());
  });
}

// Do this
function Good() {
  const price = fx(100);
  const quantity = fx(2);
  const total = () => price() * quantity(); // Derived value
}
```

### 2. Keep Run Simple

Run should handle one specific side effect:

```tsx
// Too much in one run
run(() => {
  localStorage.setItem("user", user());
  document.title = user().name;
  analytics.track("user_update", user());
});

// Separate concerns
run(() => localStorage.setItem("user", user()));
run(() => document.title = user().name);
run(() => analytics.track("user_update", user()));
```

### 3. Avoid Circular Dependencies

Don't create run that update fx they read from:

```tsx
// This creates an infinite loop
const count = fx(0);
run(() => {
  count.set(count() + 1); // Never do this!
});
```
