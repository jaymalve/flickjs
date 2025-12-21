# Flick

A tiny reactive JS framework from the future

## What is Flick?

Flick is a lightweight, reactive UI framework that combines the simplicity of modern reactive programming with the performance of direct DOM manipulation. At just around a KB minified, it provides a powerful reactivity system inspired from existing frameworks.

### Key Features

- **Fine-grained Reactivity**: Updates only what changed, no diffing required
- **No Virtual DOM**: Direct DOM manipulation for maximum performance
- **Tiny Size**: ~300 bytes minified runtime
- **JSX Support**: Write components with familiar JSX syntax
- **Zero Dependencies**: No framework overhead in production
- **TypeScript Ready**: Full TypeScript support

## Quick Start

The fastest way to get started is using the CLI:

```bash
bun create flickdev-app my-app
cd my-app
bun install
bun dev
```

## Example

```tsx
import { signal, effect, mount } from "@flickjs/runtime";

function Counter() {
  const count = signal(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={() => count.set(count() + 1)}>Increment</button>
    </div>
  );
}

mount(Counter, document.getElementById("app"));
```

## How It Works

Flick uses a compile-time approach to reactivity:

1. **Write JSX** - Use familiar component syntax
2. **Compile** - The Babel plugin transforms JSX into vanilla JS with reactive bindings
3. **Run** - The tiny runtime tracks dependencies and updates the DOM

### Reactivity System

Flick's reactivity is based on three core primitives:

- **`signal(value)`** - Create a reactive value
- **`effect(fn)`** - Run code when signals change
- **`mount(Component, element)`** - Mount a component to the DOM

The compiler automatically wraps reactive expressions in `effect()` calls, so you get fine-grained updates without manual tracking.

## Packages

This monorepo contains three packages:

### [@flickjs/runtime](./packages/runtime)

The reactive runtime library provides `signal`, `effect`, and `mount`.

```bash
bun add @flickjs/runtime
```

[View Runtime Documentation →](./packages/runtime/README.md)

### [@flickjs/compiler](./packages/compiler)

Babel plugin to transform JSX into vanilla JavaScript with Flick reactivity.

```bash
bun add -d @flickjs/compiler @babel/core
```

[View Compiler Documentation →](./packages/compiler/README.md)

### [create-flickdev-app](./packages/create-app)

CLI tool for scaffolding new Flick projects.

```bash
bun create flickdev-app my-app
```

[View CLI Documentation →](./packages/create-app/README.md)

## Manual Setup

If you prefer to set up a project manually:

```bash
# Create project
mkdir my-flick-app && cd my-flick-app
bun init -y

# Install dependencies
bun add @flickjs/runtime
bun add -d @flickjs/compiler @babel/core

# Create babel.config.js
echo 'export default { plugins: ["@flickjs/compiler"] }' > babel.config.js

# Create src/main.tsx
mkdir src
cat > src/main.tsx << 'EOF'
import { signal, mount } from '@flickjs/runtime';

function App() {
  const count = signal(0);

  return (
    <div>
      <h1>Count: {count()}</h1>
      <button onclick={() => count.set(count() + 1)}>+</button>
    </div>
  );
}

mount(App, document.getElementById('app'));
EOF

# Create index.html
cat > index.html << 'EOF'
<!doctype html>
<html>
  <head><title>Flick App</title></head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
EOF

# Build and serve
bun build src/main.tsx --outdir dist
```

## Comparison to Other Frameworks

| Feature        | Flick        | SolidJS      | Vue 3        | React          |
| -------------- | ------------ | ------------ | ------------ | -------------- |
| Size           | ~300 bytes   | ~7 KB        | ~34 KB       | ~45 KB         |
| Reactivity     | Fine-grained | Fine-grained | Fine-grained | Coarse-grained |
| Virtual DOM    | No           | No           | Optional     | Yes            |
| JSX Support    | Yes          | Yes          | Via plugin   | Yes            |
| Learning Curve | Low          | Medium       | Medium       | Medium         |

Flick is most similar to **SolidJS** in its reactivity model, but with an even smaller footprint and simpler API.

## Advanced Examples

### Computed Values

```tsx
function TodoList() {
  const todos = signal([
    { id: 1, text: "Learn Flick", done: false },
    { id: 2, text: "Build an app", done: false },
  ]);

  const remaining = signal(0);

  effect(() => {
    remaining.set(todos().filter((t) => !t.done).length);
  });

  return (
    <div>
      <h2>Remaining: {remaining()}</h2>
      <ul>
        {todos().map((todo) => (
          <li>{todo.text}</li>
        ))}
      </ul>
    </div>
  );
}
```

### Event Handling

```tsx
function Form() {
  const name = signal("");
  const email = signal("");

  const handleSubmit = (e) => {
    e.preventDefault();
    console.log("Submitted:", { name: name(), email: email() });
  };

  return (
    <form onsubmit={handleSubmit}>
      <input
        type="text"
        value={name()}
        oninput={(e) => name.set(e.target.value)}
      />
      <input
        type="email"
        value={email()}
        oninput={(e) => email.set(e.target.value)}
      />
      <button type="submit">Submit</button>
    </form>
  );
}
```

## Development

This project uses Bun as a monorepo with workspaces.

### Setup

```bash
git clone https://github.com/jaymalave/flick.git
cd flick
bun install
```

### Build

```bash
# Build all packages
bun run build

# Build specific package
bun run build:runtime
bun run build:compiler
bun run build:create-app

# Clean all build outputs
bun run clean
```

### Test Create App Locally

```bash
cd packages/create-app
bun run dist/index.js test-project
cd test-project
bun install
bun dev
```

### Publishing

```bash
# Dry run (test without publishing)
bun run publish:dry

# Publish all packages to npm
bun run publish:all
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

1. Fork the repository
2. Create your feature branch
3. Make your changes
4. Run `bun run build` to ensure everything builds
5. Submit a pull request

## License

MIT © Jay Malave

## Links

- [GitHub Repository](https://github.com/jaymalave/flick)
- [Issues](https://github.com/jaymalave/flick/issues)
- [npm: @flickjs/runtime](https://www.npmjs.com/package/@flickjs/runtime)
- [npm: @flickjs/compiler](https://www.npmjs.com/package/@flickjs/compiler)
- [npm: create-flickdev-app](https://www.npmjs.com/package/create-flickdev-app)
