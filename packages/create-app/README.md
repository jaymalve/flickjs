# create-flickdev-app

Scaffolding CLI tool for creating new Flick projects.

## Quick Start

Create a new Flick app with a single command:

```bash
bun create flickdev-app my-app
```

Or with npm:

```bash
npm create flickdev-app my-app
```

Or with npx:

```bash
npx create-flickdev-app my-app
```

Then navigate to your new project and start developing:

```bash
cd my-app
bun install
bun dev
```

## What It Creates

The CLI generates a minimal Flick project with the following structure:

```
my-app/
├── index.html          # Entry HTML file
└── src/
    └── main.tsx        # Main application file
```

### index.html

A simple HTML file with a mount point and module script:

```html
<!DOCTYPE html>
<div id="app"></div>
<script type="module" src="/src/main.tsx"></script>
```

### src/main.tsx

A basic counter example to get you started:

```tsx
import { mount } from "@flickjs/runtime";

function App() {
  let count = 0;

  return <button onclick={() => count++}>Count: {count}</button>;
}

mount(App, document.getElementById("app"));
```

## Usage

### Basic Usage

Create an app in a new directory:

```bash
bun create flickdev-app my-app
```

This creates a directory called `my-app` in the current location.

### Custom Project Name

You can specify any project name:

```bash
bun create flickdev-app todo-app
bun create flickdev-app my-awesome-project
```

### Current Directory

To create a project in the current directory, use `.`:

```bash
mkdir my-project
cd my-project
bun create flickdev-app .
```

## Next Steps

After creating your app, follow these steps:

### 1. Install Dependencies

```bash
cd my-app
bun install
```

This installs:

- `@flickjs/runtime` - The reactive runtime
- `@flickjs/compiler` - The JSX compiler
- `@babel/core` - Babel for JSX transformation

### 2. Set Up Development Server

You can use any development server. Here are a few options:

#### Option A: Bun (Recommended)

Add to your `package.json`:

```json
{
  "scripts": {
    "dev": "bun --hot src/main.tsx"
  }
}
```

Then run:

```bash
bun dev
```

#### Option B: Vite

Install Vite and the Babel plugin:

```bash
bun add -d vite vite-plugin-babel
```

Create `vite.config.js`:

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

Add scripts to `package.json`:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }
}
```

#### Option C: Simple HTTP Server

For a simple static server:

```bash
bun add -d http-server
```

Add to `package.json`:

```json
{
  "scripts": {
    "dev": "http-server -o"
  }
}
```

### 3. Build for Production

Add a build script to your `package.json`:

```json
{
  "scripts": {
    "build": "bun build src/main.tsx --outdir dist --minify"
  }
}
```

Then build:

```bash
bun run build
```

### 4. Configure TypeScript (Optional)

If you want full TypeScript support, create a `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "@flickjs/runtime",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "types": ["bun-types"]
  },
  "include": ["src/**/*"]
}
```

## Project Examples

### Expanding the Counter

Edit `src/main.tsx` to add more features:

```tsx
import { signal, mount } from "@flickjs/runtime";

function App() {
  const count = signal(0);

  const increment = () => count.set(count() + 1);
  const decrement = () => count.set(count() - 1);
  const reset = () => count.set(0);

  return (
    <div>
      <h1>Counter: {count()}</h1>
      <button onclick={decrement}>-</button>
      <button onclick={reset}>Reset</button>
      <button onclick={increment}>+</button>
    </div>
  );
}

mount(App, document.getElementById("app"));
```

### Todo List

```tsx
import { signal, mount } from "@flickjs/runtime";

function App() {
  const todos = signal([{ id: 1, text: "Learn Flick", done: false }]);
  const input = signal("");

  const addTodo = () => {
    if (input().trim()) {
      todos.set([...todos(), { id: Date.now(), text: input(), done: false }]);
      input.set("");
    }
  };

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
      <div>
        <input
          value={input()}
          oninput={(e) => input.set(e.target.value)}
          placeholder="Add a todo"
        />
        <button onclick={addTodo}>Add</button>
      </div>
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

mount(App, document.getElementById("app"));
```

## Troubleshooting

### Module not found: @flickjs/runtime

Make sure you've installed dependencies:

```bash
bun install
```

### JSX transform not working

Ensure you have a `babel.config.js` file:

```javascript
export default {
  plugins: ["@flickjs/compiler"],
};
```

### TypeScript errors with JSX

Add to your `tsconfig.json`:

```json
{
  "compilerOptions": {
    "jsx": "preserve"
  }
}
```

## Comparison with Other CLIs

| Feature      | create-flickdev-app  | create-react-app | create-solid  |
| ------------ | -------------------- | ---------------- | ------------- |
| Bundle size  | Minimal (~300 bytes) | Large (~45 KB)   | Small (~7 KB) |
| Setup time   | Instant              | ~2 minutes       | ~30 seconds   |
| Config files | Optional             | Many             | Some          |
| Dependencies | 2                    | 1000+            | 50+           |
| Build tool   | Any                  | webpack          | Vite          |

## Advanced Configuration

### Custom Babel Config

Create a `babel.config.js` with additional plugins:

```javascript
export default {
  plugins: [
    "@flickjs/compiler",
    // Add more Babel plugins here
  ],
  presets: [
    // Add Babel presets here
  ],
};
```

### Adding CSS

Add a `<link>` to your `index.html`:

```html
<!DOCTYPE html>
<html>
  <head>
    <link rel="stylesheet" href="/src/style.css" />
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

Or import CSS in your component (if your bundler supports it):

```tsx
import "./style.css";
```

### Multiple Pages

Create additional entry points:

```
src/
├── main.tsx       # Home page
├── about.tsx      # About page
└── contact.tsx    # Contact page
```

And corresponding HTML files:

```html
<!-- index.html -->
<script type="module" src="/src/main.tsx"></script>

<!-- about.html -->
<script type="module" src="/src/about.tsx"></script>
```

## Links

- [Main Documentation](../../README.md)
- [Runtime Documentation](../runtime/README.md)
- [Compiler Documentation](../compiler/README.md)
- [GitHub Repository](https://github.com/jaymalave/flick)
- [npm Package](https://www.npmjs.com/package/create-flickdev-app)

## Support

If you encounter any issues:

1. Check the [GitHub Issues](https://github.com/jaymalave/flick/issues)
2. Create a new issue with details about your problem
3. Include your OS, Node/Bun version, and error messages

## License

MIT © Jay Malave
