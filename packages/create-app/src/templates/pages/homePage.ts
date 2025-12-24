import fs from "fs-extra";
import path from "path";

export function createHomePage(root: string) {
  fs.writeFileSync(
    path.join(root, "src/pages/index.tsx"),
    `import { signal, effect } from "@flickjs/runtime";
import { Link } from "@flickjs/router";

export default function Home() {
  const count = signal(0);

  effect(() => {
    console.log("Count changed:", count());
  });

  return (
    <div class="min-h-screen bg-gray-100 p-8">
      <div class="max-w-4xl mx-auto">
        <h1 class="text-4xl font-bold text-gray-800 mb-6">Welcome to Flick</h1>

        <div class="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 class="text-2xl font-semibold mb-4">Signal & Effect Example</h2>
          <p class="text-xl mb-4">Count: {count()}</p>
          <div class="space-x-2">
            <button
              onclick={() => count.set(count() + 1)}
              class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
            >
              Increment
            </button>
            <button
              onclick={() => count.set(count() - 1)}
              class="bg-gray-500 hover:bg-gray-600 text-white px-4 py-2 rounded"
            >
              Decrement
            </button>
            <button
              onclick={() => count.set(0)}
              class="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded"
            >
              Reset
            </button>
          </div>
        </div>

        <nav class="bg-white rounded-lg shadow-md p-6">
          <h2 class="text-2xl font-semibold mb-4">Navigation</h2>
          <div class="space-x-4">
            <Link href="/todos" class="text-blue-500 hover:underline">
              Todo List (Resource Examples)
            </Link>
            <Link href="/todos/1" class="text-blue-500 hover:underline">
              Todo #1 (Dynamic Route)
            </Link>
            <Link href="/about" class="text-blue-500 hover:underline">
              About (Lazy Loading)
            </Link>
          </div>
        </nav>
      </div>
    </div>
  );
}
`
  );
}
