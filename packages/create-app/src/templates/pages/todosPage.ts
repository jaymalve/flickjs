import fs from "fs-extra";
import path from "path";

export function createTodosPage(root: string) {
  fs.writeFileSync(
    path.join(root, "src/pages/todos.tsx"),
    `import { signal, resource, Suspense } from "@flickjs/runtime";
import { Link } from "@flickjs/router";

interface Todo {
  userId: number;
  id: number;
  title: string;
  completed: boolean;
}

export default function TodoList() {
  // Simple resource - fetches all todos (limited to 10)
  const allTodos = resource<Todo[]>(() =>
    fetch("https://jsonplaceholder.typicode.com/todos?_limit=10")
      .then(r => r.json())
  );

  // Signal for userId filter
  const selectedUserId = signal(1);

  // Source-based resource - refetches when selectedUserId changes
  const userTodos = resource<Todo[], number>(
    () => selectedUserId(),
    (userId) => fetch(\`https://jsonplaceholder.typicode.com/todos?userId=\${userId}\`)
      .then(r => r.json())
  );

  return (
    <div class="min-h-screen bg-gray-100 p-8">
      <div class="max-w-4xl mx-auto">
        <div class="mb-6">
          <Link href="/" class="text-blue-500 hover:underline">
            ← Back to Home
          </Link>
        </div>

        <h1 class="text-4xl font-bold text-gray-800 mb-6">Todo List Examples</h1>

        {/* Simple Resource Example */}
        <div class="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 class="text-2xl font-semibold mb-4">Simple Resource (First 10 Todos)</h2>
          <Suspense fallback={() => <div class="text-gray-500">Loading todos...</div>}>
            {() => {
              const todos = allTodos();
              if (!todos) return <div>No data</div>;
              return (
                <ul class="space-y-2">
                  {todos.map((todo: Todo) => (
                    <li class="flex items-center gap-2">
                      <span class={\`\${todo.completed ? "line-through text-gray-400" : ""}\`}>
                        {todo.title}
                      </span>
                      <Link href={\`/todos/\${todo.id}\`} class="text-blue-500 text-sm hover:underline">
                        View
                      </Link>
                    </li>
                  ))}
                </ul>
              );
            }}
          </Suspense>
        </div>

        {/* Source-based Resource Example */}
        <div class="bg-white rounded-lg shadow-md p-6">
          <h2 class="text-2xl font-semibold mb-4">Source-based Resource (Filter by User)</h2>

          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-2">
              Select User ID:
            </label>
            <select
              class="border border-gray-300 rounded px-3 py-2"
              onchange={(e: Event) => selectedUserId.set(Number((e.target as HTMLSelectElement).value))}
            >
              <option value="1">User 1</option>
              <option value="2">User 2</option>
              <option value="3">User 3</option>
              <option value="4">User 4</option>
              <option value="5">User 5</option>
            </select>
          </div>

          <Suspense fallback={() => <div class="text-gray-500">Loading user todos...</div>}>
            {() => {
              const todos = userTodos();
              if (!todos) return <div>No data</div>;
              return (
                <div>
                  <p class="text-sm text-gray-600 mb-2">
                    Showing {todos.length} todos for User {selectedUserId()}
                  </p>
                  <ul class="space-y-2">
                    {todos.map((todo: Todo) => (
                      <li class="flex items-center gap-2">
                        <input type="checkbox" checked={todo.completed} disabled />
                        <span>{todo.title}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              );
            }}
          </Suspense>
        </div>
      </div>
    </div>
  );
}
`
  );
}
