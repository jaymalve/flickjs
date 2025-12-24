import fs from "fs-extra";
import path from "path";

export function createIndexCss(root: string) {
  fs.writeFileSync(
    path.join(root, "src/index.css"),
    `@import "tailwindcss";
`
  );
}
