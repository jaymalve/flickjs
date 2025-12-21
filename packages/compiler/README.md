# @flickjs/compiler

Babel plugin to compile JSX to vanilla JavaScript with Flick reactivity.

## Installation

```bash
bun add -d @flickjs/compiler @babel/core
# or
npm install --save-dev @flickjs/compiler @babel/core
```

## What It Does

The Flick compiler transforms JSX into vanilla JavaScript DOM operations with fine-grained reactive bindings. Unlike React's JSX, which creates a virtual DOM, Flick's JSX compiles directly to imperative DOM manipulation code that updates only when signals change.

## Configuration

### Babel Config

Create a `babel.config.js` file:

```javascript
export default {
  plugins: ["@flickjs/compiler"],
};
```

Or in CommonJS format:

```javascript
module.exports = {
  plugins: ["@flickjs/compiler"],
};
```

### With Bun

Bun has built-in Babel support. Just add the babel config file and build:

```bash
bun build src/main.tsx --outdir dist
```

### With Vite

Install the Babel plugin for Vite:

```bash
bun add -d vite-plugin-babel
```

Then in `vite.config.js`:

```javascript
import { defineConfig } from "vite";
import babel from "vite-plugin-babel";

export default defineConfig({
  plugins: [
    babel({
      babelConfig: {
        plugins: ["@flickjs/compiler"],
      },
    }),
  ],
});
```

### With webpack

Install babel-loader:

```bash
npm install --save-dev babel-loader
```

Then in `webpack.config.js`:

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.(js|jsx|ts|tsx)$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
          options: {
            plugins: ["@flickjs/compiler"],
          },
        },
      },
    ],
  },
};
```

## How It Works

The compiler transforms JSX elements into vanilla DOM operations and wraps reactive expressions in `effect()` calls.

### Transformation Examples

#### Static Elements

**Input**:

```jsx
<div class="container">
  <h1>Hello World</h1>
</div>
```

**Output**:

```javascript
(() => {
  const el = document.createElement("div");
  el.className = "container";
  const child = document.createElement("h1");
  child.textContent = "Hello World";
  el.append(child);
  return el;
})();
```

#### Reactive Text

**Input**:

```jsx
<h1>Count: {count()}</h1>
```

**Output**:

```javascript
(() => {
  const el = document.createElement("h1");
  const text1 = document.createTextNode("Count: ");
  const text2 = document.createTextNode("");
  el.append(text1, text2);

  effect(() => {
    text2.data = count();
  });

  return el;
})();
```

#### Event Handlers

**Input**:

```jsx
<button onclick={() => count.set(count() + 1)}>Click me</button>
```

**Output**:

```javascript
(() => {
  const el = document.createElement("button");
  el.onclick = () => count.set(count() + 1);
  el.textContent = "Click me";
  return el;
})();
```

#### Reactive Attributes

**Input**:

```jsx
<input value={name()} oninput={(e) => name.set(e.target.value)} />
```

**Output**:

```javascript
(() => {
  const el = document.createElement("input");

  effect(() => {
    el.value = name();
  });

  el.oninput = (e) => name.set(e.target.value);

  return el;
})();
```

#### Component Calls

**Input**:

```jsx
function Button({ label }) {
  return <button>{label}</button>;
}

function App() {
  return <Button label="Click" />;
}
```

**Output**:

```javascript
function Button({ label }) {
  const el = document.createElement("button");
  el.textContent = label;
  return el;
}

function App() {
  return Button({ label: "Click" });
}
```

## TypeScript Configuration

To use JSX with TypeScript, add to your `tsconfig.json`:

```json
{
  "compilerOptions": {
    "jsx": "preserve",
    "jsxImportSource": "@flickjs/runtime"
  }
}
```

Or use the classic JSX transform:

```json
{
  "compilerOptions": {
    "jsx": "preserve"
  }
}
```

## Features

### Supported JSX Features

- ✅ Elements: `<div>`, `<span>`, etc.
- ✅ Attributes: `class`, `id`, `style`, etc.
- ✅ Event handlers: `onclick`, `oninput`, etc.
- ✅ Children: text, elements, arrays
- ✅ Components: function components
- ✅ Reactive expressions: `{count()}`
- ✅ Fragments: `<>...</>`

### Not Supported

- ❌ Class components (use function components)
- ❌ Lifecycle methods (use effects)
- ❌ Context API (pass props)
- ❌ Refs (use direct DOM manipulation)

## Advanced Usage

### Conditional Rendering

Since JSX compiles to functions, you can use regular JavaScript for conditionals:

```tsx
function Greeting({ isLoggedIn }) {
  return (
    <div>{isLoggedIn ? <h1>Welcome back!</h1> : <h1>Please sign in</h1>}</div>
  );
}
```

Or with signals:

```tsx
function Greeting() {
  const isLoggedIn = signal(false);

  return (
    <div>{isLoggedIn() ? <h1>Welcome back!</h1> : <h1>Please sign in</h1>}</div>
  );
}
```

### Lists

Use `.map()` to render lists:

```tsx
function TodoList() {
  const todos = signal([
    { id: 1, text: "Learn Flick" },
    { id: 2, text: "Build app" },
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

### Fragments

Use fragments to return multiple elements:

```tsx
function Header() {
  return (
    <>
      <h1>Title</h1>
      <p>Subtitle</p>
    </>
  );
}
```

### Style Objects

You can pass style objects directly:

```tsx
function StyledDiv() {
  const color = signal("red");

  return <div style={{ color: color(), fontSize: "16px" }}>Styled text</div>;
}
```

## Performance

The compiler generates optimal code:

- **Zero runtime overhead** for static content
- **Minimal reactivity cost** - only reactive expressions are tracked
- **No virtual DOM** - direct DOM manipulation
- **Tree-shakeable** - unused code is eliminated

## Debugging

### Source Maps

When building with source maps, you can debug the original JSX code:

```bash
bun build src/main.tsx --outdir dist --sourcemap
```

### Compiler Output

To see the compiled output, use Babel CLI:

```bash
npx babel src/main.tsx --plugins @flickjs/compiler
```

## Common Patterns

### Form Handling

```tsx
function LoginForm() {
  const email = signal("");
  const password = signal("");

  const handleSubmit = (e) => {
    e.preventDefault();
    console.log({ email: email(), password: password() });
  };

  return (
    <form onsubmit={handleSubmit}>
      <input
        type="email"
        value={email()}
        oninput={(e) => email.set(e.target.value)}
        placeholder="Email"
      />
      <input
        type="password"
        value={password()}
        oninput={(e) => password.set(e.target.value)}
        placeholder="Password"
      />
      <button type="submit">Login</button>
    </form>
  );
}
```

### Dynamic Classes

```tsx
function Button({ active }) {
  const isActive = signal(active);

  return (
    <button
      class={isActive() ? "btn btn-active" : "btn"}
      onclick={() => isActive.set(!isActive())}
    >
      Toggle
    </button>
  );
}
```

### Composition

```tsx
function Card({ title, children }) {
  return (
    <div class="card">
      <h2>{title}</h2>
      <div class="card-body">{children}</div>
    </div>
  );
}

function App() {
  return (
    <Card title="Hello">
      <p>This is the card content</p>
    </Card>
  );
}
```

## Migration from React

If you're coming from React:

- **Components**: Use function components (same as React)
- **State**: Replace `useState` with `signal`
- **Effects**: Replace `useEffect` with `effect`
- **Props**: Works the same way
- **Events**: Use lowercase names (`onclick` not `onClick`)
- **Class**: Use `class` not `className`
- **Style**: Can use objects (same as React)

**React**:

```tsx
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}
```

**Flick**:

```tsx
function Counter() {
  const count = signal(0);
  return <button onclick={() => count.set(count() + 1)}>{count()}</button>;
}
```

## Links

- [Main Documentation](../../README.md)
- [Runtime Documentation](../runtime/README.md)
- [CLI Documentation](../create-app/README.md)
- [GitHub Repository](https://github.com/jaymalave/flick)
- [npm Package](https://www.npmjs.com/package/@flickjs/compiler)

## License

MIT © Jay Malave
