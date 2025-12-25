# Installation

## Manual Installation

If you want to add Flick to an existing project:

```bash
# Install runtime (required)
bun add @flickjs/runtime

# Install vite plugin (required for JSX)
bun add -D @flickjs/vite-plugin

# Install router (optional)
bun add @flickjs/router
```

## Vite Configuration

Create or update `vite.config.js`:

```js
import { defineConfig } from "vite";
import flick from "@flickjs/vite-plugin";

export default defineConfig({
  plugins: [flick()],
});
```

## HTML Entry Point

Create an `index.html`:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>My Flick App</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

## Main Entry Point

Create `src/main.tsx`:

```tsx
import { fx, mount } from "@flickjs/runtime";

function App() {
  const count = fx(0);

  return (
    <div>
      <h1>Hello Flick!</h1>
      <p>Count: {count()}</p>
      <button onclick={() => count.set(count() + 1)}>Increment</button>
    </div>
  );
}

mount(App, document.getElementById("app"));
```

## TypeScript Configuration

Add a `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*"]
}
```

## Package Scripts

Add scripts to your `package.json`:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }
}
```
