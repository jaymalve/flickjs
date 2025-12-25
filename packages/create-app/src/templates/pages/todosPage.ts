import fs from "fs-extra";
import path from "path";

export function createTodosPage(root: string) {
  fs.writeFileSync(
    path.join(root, "src/pages/todos.tsx"),
    `import { fx, query, Suspense } from "@flickjs/runtime";
import { Link } from "@flickjs/router";
import TodoChip from "../components/TodoChip";
import type { Todo } from "../types/todo.interface";

export default function TodoList() {
  const handleFetchAllTodos = async () => {
    const res = await fetch(
      "https://jsonplaceholder.typicode.com/todos?_limit=10"
    );
    const response = await res.json();
    console.log("response", response);
    return response;
  };
  const allTodos = query(handleFetchAllTodos);

  // Fx for userId filter
  const selectedUserId = fx(1);

  const handleFetchUserTodos = async (userId: number) => {
    const res = await fetch(
      \`https://jsonplaceholder.typicode.com/todos?userId=\${userId}\`
    );
    const response = await res.json();
    console.log("response", response);
    return response;
  };

  const userTodos = query(selectedUserId, handleFetchUserTodos);

  return (
    <div class="min-h-screen bg-gray-100 p-8">
      <div class="max-w-4xl mx-auto">
        <div class="mb-6">
          <Link href="/" class="text-blue-500 hover:underline">
            ← Back to Home
          </Link>
        </div>

        <h1 class="text-4xl font-bold text-gray-800 mb-6">
          Todo List Examples
        </h1>

        {/* Simple Query Example */}
        <div class="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 class="text-2xl font-semibold mb-4">
            Simple Query (First 10 Todos)
          </h2>
          <Suspense
            fallback={<div class="text-gray-500">Loading todos...</div>}
          >
            <ul class="space-y-2">
              {allTodos().map((todo: Todo) => (
                <li class="flex items-center gap-2">
                  <TodoChip todo={todo} onClick={() => {}} />
                </li>
              ))}
            </ul>
          </Suspense>
        </div>

        {/* Source-based Query Example */}
        <div class="bg-white rounded-lg shadow-md p-6">
          <h2 class="text-2xl font-semibold mb-4">
            Source-based Query (Filter by User)
          </h2>

          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-2">
              Select User ID:
            </label>
            <select
              class="border border-gray-300 rounded px-3 py-2"
              onchange={(e: Event) =>
                selectedUserId.set(
                  Number((e.target as HTMLSelectElement).value)
                )
              }
            >
              <option value="1">User 1</option>
              <option value="2">User 2</option>
              <option value="3">User 3</option>
              <option value="4">User 4</option>
              <option value="5">User 5</option>
            </select>
          </div>

          <Suspense
            fallback={() => (
              <div class="text-gray-500">Loading user todos...</div>
            )}
          >
            <div>
              <p class="text-sm text-gray-600 mb-2">
                Showing {userTodos().length} todos for User {selectedUserId()}
              </p>
              <ul class="space-y-2">
                {userTodos().map((todo: Todo) => (
                  <li class="flex items-center gap-2">
                    <input type="checkbox" checked={todo.completed} disabled />
                    <span>{todo.title}</span>
                  </li>
                ))}
              </ul>
            </div>
          </Suspense>
        </div>
      </div>
    </div>
  );
}
`
  );
}
