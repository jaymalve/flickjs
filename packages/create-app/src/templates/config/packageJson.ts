import fs from "fs-extra";
import path from "path";

export function createPackageJson(root: string, name: string) {
  fs.writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify(
      {
        name: name,
        version: "0.1.0",
        type: "module",
        scripts: {
          dev: "bunx --bun vite",
          build: "bunx --bun vite build",
          preview: "bunx --bun vite preview",
        },
        dependencies: {
          "@flickjs/runtime": "^0.0.1-beta.2",
          "@flickjs/router": "^0.0.1-beta.1",
        },
        devDependencies: {
          "@babel/core": "^7.24.0",
          "@flickjs/compiler": "^0.0.1-beta.2",
          "@tailwindcss/vite": "^4.1.18",
          tailwindcss: "^4",
          vite: "^5.0.0",
        },
      },
      null,
      2
    ) + "\n"
  );
}
