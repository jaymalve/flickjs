import fs from "fs-extra";
import path from "path";

export function createTodoInterface(root: string) {
  fs.writeFileSync(
    path.join(root, "src/types/todo.interface.ts"),
    `export interface Todo {
  userId: number;
  id: number;
  title: string;
  completed: boolean;
}
`
  );
}
