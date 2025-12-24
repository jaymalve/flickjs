import fs from "fs-extra";
import path from "path";

export function createTodoChip(root: string) {
  fs.writeFileSync(
    path.join(root, "src/components/TodoChip.tsx"),
    `import type { Todo } from "../types/todo.interface";

interface TodoChipProps {
  todo: Todo;
  onClick: () => void;
}

export default function TodoChip({ todo, onClick }: TodoChipProps) {
  return (
    <div
      class="flex items-center gap-2 bg-gray-100 p-2 rounded-md"
      onclick={onClick}
    >
      <span class={\`\${todo.completed ? "line-through text-gray-400" : ""}\`}>
        {todo.title}
      </span>
      <span class="text-sm text-gray-400">{todo.userId}</span>
      <span class="text-sm text-gray-400">{todo.id}</span>
    </div>
  );
}
`
  );
}
