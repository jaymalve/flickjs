import fs from "fs-extra";
import path from "path";

export function createGitignore(root: string) {
  fs.writeFileSync(
    path.join(root, ".gitignore"),
    `node_modules/
dist/
.DS_Store
*.log
`
  );
}
