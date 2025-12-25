# @flickjs/runtime

A tiny reactive runtime library with fine-grained reactivity.

## Installation

```bash
bun add @flickjs/runtime
# or
npm install @flickjs/runtime
```

## Size

- **Minified**: ~300 bytes
- **Gzipped**: ~200 bytes

The entire reactive runtime is incredibly lightweight, making it perfect for performance-critical applications.

## API

### `fx<T>(initialValue: T)`

Creates a reactive fx that holds a value. Fx are the foundation of Flick's reactivity system.

**Returns**: An object with:

- A getter function `()` to read the current value
- A `set(newValue)` method to update the value

**Example**:

```tsx
import { fx } from "@flickjs/runtime";

const count = fx(0);

console.log(count()); // 0
count.set(5);
console.log(count()); // 5

// Updating based on previous value
count.set(count() + 1);
console.log(count()); // 6
```

### `run(fn: () => void)`

Creates a run that automatically tracks fx dependencies and re-runs when they change.

**Parameters**:

- `fn`: A function that will be executed immediately and re-executed whenever any fx it reads changes

**Example**:

```tsx
import { fx, run } from "@flickjs/runtime";

const count = fx(0);

run(() => {
  console.log("Count is:", count());
});
// Logs: "Count is: 0"

count.set(5);
// Logs: "Count is: 5"
```

**Note**: In most cases, you won't need to use `run()` directly. The Flick compiler automatically wraps reactive expressions in JSX with run calls.

### `mount(Component: () => Element, target: Element)`

Mounts a Flick component to a DOM element.

**Parameters**:

- `Component`: A function that returns a DOM element
- `target`: The DOM element to mount the component to

**Example**:

```tsx
import { fx, mount } from "@flickjs/runtime";

function App() {
  const count = fx(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={() => count.set(count() + 1)}>Increment</button>
    </div>
  );
}

mount(App, document.getElementById("app"));
```

## Complete Examples

### Counter

```tsx
import { fx, mount } from "@flickjs/runtime";

function Counter() {
  const count = fx(0);

  const increment = () => count.set(count() + 1);
  const decrement = () => count.set(count() - 1);
  const reset = () => count.set(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={decrement}>-</button>
      <button onclick={reset}>Reset</button>
      <button onclick={increment}>+</button>
    </div>
  );
}

mount(Counter, document.getElementById("app"));
```

### Input Binding

```tsx
import { fx, mount } from "@flickjs/runtime";

function NameInput() {
  const name = fx("");

  return (
    <div>
      <input
        type="text"
        value={name()}
        oninput={(e) => name.set(e.target.value)}
        placeholder="Enter your name"
      />
      <p>Hello, {name() || "stranger"}!</p>
    </div>
  );
}

mount(NameInput, document.getElementById("app"));
```

### Computed Values

```tsx
import { fx, run, mount } from "@flickjs/runtime";

function TodoApp() {
  const todos = fx([
    { id: 1, text: "Learn Flick", done: false },
    { id: 2, text: "Build something", done: false },
    { id: 3, text: "Ship it", done: false },
  ]);

  const remaining = fx(0);
  const total = fx(0);

  // Computed values using run
  run(() => {
    const list = todos();
    total.set(list.length);
    remaining.set(list.filter((t) => !t.done).length);
  });

  const toggleTodo = (id) => {
    todos.set(
      todos().map((todo) =>
        todo.id === id ? { ...todo, done: !todo.done } : todo
      )
    );
  };

  return (
    <div>
      <h1>Todo List</h1>
      <p>
        {remaining()} of {total()} remaining
      </p>
      <ul>
        {todos().map((todo) => (
          <li
            onclick={() => toggleTodo(todo.id)}
            style={{ textDecoration: todo.done ? "line-through" : "none" }}
          >
            {todo.text}
          </li>
        ))}
      </ul>
    </div>
  );
}

mount(TodoApp, document.getElementById("app"));
```

### Multiple Signals

```tsx
import { fx, run, mount } from "@flickjs/runtime";

function Calculator() {
  const a = fx(0);
  const b = fx(0);
  const result = fx(0);

  run(() => {
    result.set(a() + b());
  });

  return (
    <div>
      <input
        type="number"
        value={a()}
        oninput={(e) => a.set(Number(e.target.value))}
      />
      <span> + </span>
      <input
        type="number"
        value={b()}
        oninput={(e) => b.set(Number(e.target.value))}
      />
      <span> = {result()}</span>
    </div>
  );
}

mount(Calculator, document.getElementById("app"));
```

## Advanced Patterns

### Cleanup

Effects automatically clean up and re-run when their dependencies change. No manual cleanup is needed.

### Batching Updates

Multiple signal updates are automatically batched within the same synchronous execution:

```tsx
const count = fx(0);
const doubled = fx(0);

run(() => {
  doubled.set(count() * 2);
});

// These updates are batched
count.set(5);
count.set(10);
count.set(15);
// Run executes only once with final value (15)
```

### Nested Components

You can create components that use other components:

```tsx
function Button({ label, onClick }) {
  return <button onclick={onClick}>{label}</button>;
}

function App() {
  const count = fx(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <Button label="Increment" onClick={() => count.set(count() + 1)} />
    </div>
  );
}
```

## TypeScript Support

The runtime is fully typed. Import types for better IDE support:

```tsx
import { fx, run, mount } from "@flickjs/runtime";

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

const todos = fx<Todo[]>([]);
```

## How It Works

Flick's runtime uses a simple but effective reactivity system:

1. **Fx** store values and track their dependents
2. **Run** registers themselves as dependents when they read fx
3. When an fx updates via `.set()`, it notifies all dependent run
4. Run re-executes and updates the DOM directly

This is called "fine-grained reactivity" because only the specific parts of the DOM that depend on changed fx are updated. There's no virtual DOM diffing.

## Bundle Size Impact

The runtime adds minimal overhead to your bundle:

- **Runtime**: ~300 bytes
- **Per fx**: ~50 bytes
- **Per run**: ~40 bytes

A typical small app with 10 fx and 15 run would add less than 1.5 KB to your bundle.

## Browser Support

Flick works in all modern browsers that support:

- ES6 modules
- Arrow functions
- Template literals

This includes:

- Chrome 61+
- Firefox 60+
- Safari 11+
- Edge 79+

## Links

- [Main Documentation](../../README.md)
- [Compiler Documentation](../compiler/README.md)
- [CLI Documentation](../create-app/README.md)
- [GitHub Repository](https://github.com/jaymalave/flick)
- [npm Package](https://www.npmjs.com/package/@flickjs/runtime)

## License

MIT © Jay Malave
