# Signals

Signals are reactive values that automatically update the UI when changed. They are the core building block of Flick's reactivity system.

## Creating Signals

```tsx
import { signal, mount } from "@flickjs/runtime";

function Counter() {
  const count = signal(0);

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

## Key Points

- `signal(initialValue)` creates a reactive signal
- `count()` reads the current value
- `count.set(newValue)` updates the value and triggers UI updates
- Only the specific DOM nodes that use the signal will update (fine-grained reactivity)

## Derived Values

Computed values that automatically update when their dependencies change:

```tsx
function PriceCalculator() {
  const price = signal(100);
  const quantity = signal(1);

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

Derived values are just functions that read from signals. They don't need any special syntax - just call them like regular functions in your JSX.

## Signal Types

Signals can hold any type of value:

```tsx
// Primitive values
const count = signal(0);
const name = signal("John");
const isActive = signal(true);

// Objects
const user = signal({ name: "John", age: 30 });

// Arrays
const todos = signal([
  { id: 1, text: "Learn Flick" },
  { id: 2, text: "Build something" },
]);
```

## Updating Objects and Arrays

When updating objects or arrays, you need to create a new reference:

```tsx
const todos = signal([{ id: 1, text: "Learn Flick" }]);

// Add item
todos.set([...todos(), { id: 2, text: "Build something" }]);

// Remove item
todos.set(todos().filter(todo => todo.id !== 1));

// Update item
todos.set(
  todos().map(todo =>
    todo.id === 1 ? { ...todo, text: "Updated" } : todo
  )
);
```

```tsx
const user = signal({ name: "John", age: 30 });

// Update property
user.set({ ...user(), age: 31 });
```

## Best Practices

1. **Keep signals at the top level** - Define signals at the beginning of your component
2. **Use derived values for computed state** - Instead of effects that update signals
3. **Prefer many small signals over one large object** - Better for fine-grained updates
