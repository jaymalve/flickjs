# Styling

Flick uses standard HTML attributes for styling, making it familiar for web developers.

## Class Attribute

Use `class` (not `className`):

```tsx
function StyledComponent() {
  const isActive = fx(false);

  return (
    <div class={isActive() ? "active" : "inactive"}>
      <button class="btn btn-primary" onclick={() => isActive.set(!isActive())}>
        Toggle
      </button>
    </div>
  );
}
```

## Inline Styles

```tsx
function ColorBox() {
  const color = fx("red");

  return (
    <div style={`background-color: ${color()}; padding: 20px;`}>
      <button onclick={() => color.set("blue")}>Make Blue</button>
      <button onclick={() => color.set("red")}>Make Red</button>
    </div>
  );
}
```

## Tailwind CSS Setup

### Installation

```bash
bun add -D tailwindcss @tailwindcss/vite
```

### Configuration

**1. Create `src/index.css`:**

```css
@import "tailwindcss";
```

**2. Update `vite.config.js`:**

```js
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
// ... your existing flickPlugin

export default defineConfig({
  plugins: [
    flickPlugin(),
    tailwindcss(),
  ],
});
```

**3. Import CSS in `src/main.tsx`:**

```tsx
import "./index.css";
import { fx, mount } from "@flickjs/runtime";

// ... rest of your app
```

### Tailwind Example

```tsx
import "./index.css";
import { fx, mount } from "@flickjs/runtime";

function Counter() {
  const count = fx(0);

  return (
    <div class="min-h-screen bg-gray-100 flex items-center justify-center">
      <div class="bg-white p-8 rounded-lg shadow-lg">
        <h1 class="text-3xl font-bold text-gray-800 mb-4">
          Count: {count()}
        </h1>
        <div class="flex gap-2">
          <button
            class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
            onclick={() => count.set(count() + 1)}
          >
            Increment
          </button>
          <button
            class="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded"
            onclick={() => count.set(count() - 1)}
          >
            Decrement
          </button>
        </div>
      </div>
    </div>
  );
}

mount(Counter, document.getElementById("app"));
```

## Dynamic Classes

Combine static and dynamic classes:

```tsx
function Button({ variant, children, onclick }) {
  const baseClasses = "px-4 py-2 rounded font-medium";
  const variantClasses = {
    primary: "bg-blue-500 hover:bg-blue-600 text-white",
    secondary: "bg-gray-200 hover:bg-gray-300 text-gray-800",
    danger: "bg-red-500 hover:bg-red-600 text-white",
  };

  return (
    <button
      class={`${baseClasses} ${variantClasses[variant] || variantClasses.primary}`}
      onclick={onclick}
    >
      {children}
    </button>
  );
}
```

## CSS Modules (Alternative)

If you prefer CSS Modules, you can configure Vite to use them:

```tsx
// Button.module.css
.button {
  padding: 0.5rem 1rem;
  border-radius: 0.25rem;
}

.primary {
  background: blue;
  color: white;
}
```

```tsx
// Button.tsx
import styles from "./Button.module.css";

function Button({ children }) {
  return (
    <button class={`${styles.button} ${styles.primary}`}>
      {children}
    </button>
  );
}
```
