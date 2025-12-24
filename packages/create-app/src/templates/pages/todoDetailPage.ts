import fs from "fs-extra";
import path from "path";

export function createTodoDetailPage(root: string) {
  fs.writeFileSync(
    path.join(root, "src/pages/todos/[id].tsx"),
    `import { resource, Suspense } from "@flickjs/runtime";
import { Link, params } from "@flickjs/router";

interface Todo {
  userId: number;
  id: number;
  title: string;
  completed: boolean;
}

export default function TodoDetail() {
  // Access dynamic route parameter
  const todoId = () => params().id;

  // Resource that fetches todo by ID from route params
  const todo = resource<Todo, string>(
    todoId,
    (id) => fetch(\`https://jsonplaceholder.typicode.com/todos/\${id}\`)
      .then(r => r.json())
  );

  return (
    <div class="min-h-screen bg-gray-100 p-8">
      <div class="max-w-4xl mx-auto">
        <div class="mb-6">
          <Link href="/todos" class="text-blue-500 hover:underline">
            ← Back to Todo List
          </Link>
        </div>

        <Suspense fallback={() => (
          <div class="bg-white rounded-lg shadow-md p-6">
            <div class="animate-pulse">
              <div class="h-8 bg-gray-200 rounded w-3/4 mb-4"></div>
              <div class="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
              <div class="h-4 bg-gray-200 rounded w-1/2"></div>
            </div>
          </div>
        )}>
          {() => {
            const data = todo();
            if (!data) return <div>Loading...</div>;
            return (
              <div class="bg-white rounded-lg shadow-md p-6">
                <h1 class="text-3xl font-bold text-gray-800 mb-4">
                  Todo #{data.id}
                </h1>

                <div class="space-y-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-500">Title</label>
                    <p class="text-lg">{data.title}</p>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-500">Status</label>
                    <span class={\`inline-block px-3 py-1 rounded-full text-sm \${
                      data.completed
                        ? "bg-green-100 text-green-800"
                        : "bg-yellow-100 text-yellow-800"
                    }\`}>
                      {data.completed ? "Completed" : "Pending"}
                    </span>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-500">User ID</label>
                    <p>{data.userId}</p>
                  </div>
                </div>

                <div class="mt-6 pt-4 border-t">
                  <p class="text-sm text-gray-500">
                    This page demonstrates dynamic routing with params() and
                    source-based resource fetching.
                  </p>
                </div>
              </div>
            );
          }}
        </Suspense>
      </div>
    </div>
  );
}
`
  );
}
