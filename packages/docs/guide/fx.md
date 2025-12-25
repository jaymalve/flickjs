# Fx

Fx are reactive values that automatically update the UI when changed. They are the core building block of Flick's reactivity system.

## Creating Fx

```tsx
import { fx, mount } from "@flickjs/runtime";

function Counter() {
  const count = fx(0);

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

- `fx(initialValue)` creates a reactive fx
- `count()` reads the current value
- `count.set(newValue)` updates the value and triggers UI updates
- Only the specific DOM nodes that use the fx will update (fine-grained reactivity)

## Derived Values

Computed values that automatically update when their dependencies change:

```tsx
function PriceCalculator() {
  const price = fx(100);
  const quantity = fx(1);

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

Derived values are just functions that read from fx. They don't need any special syntax - just call them like regular functions in your JSX.

## Fx Types

Fx can hold any type of value:

```tsx
// Primitive values
const count = fx(0);
const name = fx("John");
const isActive = fx(true);

// Objects
const user = fx({ name: "John", age: 30 });

// Arrays
const todos = fx([
  { id: 1, text: "Learn Flick" },
  { id: 2, text: "Build something" },
]);
```

## Updating Objects and Arrays

When updating objects or arrays, you need to create a new reference:

```tsx
const todos = fx([{ id: 1, text: "Learn Flick" }]);

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
const user = fx({ name: "John", age: 30 });

// Update property
user.set({ ...user(), age: 31 });
```

## Best Practices

1. **Keep fx at the top level** - Define fx at the beginning of your component
2. **Use derived values for computed state** - Instead of run that update fx
3. **Prefer many small fx over one large object** - Better for fine-grained updates
