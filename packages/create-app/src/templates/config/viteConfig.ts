import fs from "fs-extra";
import path from "path";

export function createViteConfig(root: string) {
  fs.writeFileSync(
    path.join(root, "vite.config.js"),
    `import { defineConfig } from "vite";
import { resolve } from "path";
import tailwindcss from "@tailwindcss/vite";
import { flickRouter } from "@flickjs/router/vite";
import flick from "@flickjs/vite-plugin";

export default defineConfig({
  plugins: [
    flick(),
    flickRouter({ pagesDir: "pages", root: resolve(__dirname) }),
    tailwindcss(),
  ],
});
`
  );
}
