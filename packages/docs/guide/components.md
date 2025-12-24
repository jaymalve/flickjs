# Components

Components in Flick are simple functions that return JSX.

## Function Components

```tsx
function Greeting({ name }) {
  return <h1>Hello, {name}!</h1>;
}

function App() {
  return (
    <div>
      <Greeting name="World" />
      <Greeting name="Flick" />
    </div>
  );
}
```

## Passing Children

```tsx
function Card({ children, title }) {
  return (
    <div class="card">
      <h2>{title}</h2>
      <div class="card-body">{children}</div>
    </div>
  );
}

function App() {
  return (
    <Card title="Welcome">
      <p>This is the card content.</p>
    </Card>
  );
}
```

## Conditional Rendering

Use JavaScript expressions for conditional rendering:

```tsx
function Toggle() {
  const visible = signal(true);

  return (
    <div>
      <button onclick={() => visible.set(!visible())}>Toggle</button>
      {visible() && <p>I'm visible!</p>}
    </div>
  );
}
```

### Ternary for Either/Or

```tsx
function Status() {
  const isOnline = signal(true);

  return (
    <div>
      {isOnline() ? <span>Online</span> : <span>Offline</span>}
    </div>
  );
}
```

## List Rendering

Use `map()` to render lists:

```tsx
function TodoList() {
  const todos = signal([
    { id: 1, text: "Learn Flick" },
    { id: 2, text: "Build something" },
  ]);

  return (
    <ul>
      {todos().map((todo) => (
        <li key={todo.id}>{todo.text}</li>
      ))}
    </ul>
  );
}
```

## Event Handling

Use lowercase event names (standard DOM events):

```tsx
function Form() {
  const value = signal("");

  return (
    <div>
      <input
        type="text"
        value={value()}
        oninput={(e) => value.set(e.target.value)}
      />
      <p>You typed: {value()}</p>

      <button onclick={() => alert("Clicked!")}>Click me</button>

      <div
        onmouseenter={() => console.log("Mouse entered")}
        onmouseleave={() => console.log("Mouse left")}
      >
        Hover over me
      </div>
    </div>
  );
}
```

## Mounting Components

Use `mount()` to render your root component:

```tsx
import { mount } from "@flickjs/runtime";

function App() {
  return <h1>Hello World</h1>;
}

mount(App, document.getElementById("app"));
```

## Component Organization

Keep components focused and reusable:

```tsx
// Button.tsx
export function Button({ children, onclick, variant = "primary" }) {
  return (
    <button class={`btn btn-${variant}`} onclick={onclick}>
      {children}
    </button>
  );
}

// App.tsx
import { Button } from "./Button";

function App() {
  return (
    <div>
      <Button onclick={() => console.log("clicked")}>Click me</Button>
      <Button variant="secondary">Secondary</Button>
    </div>
  );
}
```
