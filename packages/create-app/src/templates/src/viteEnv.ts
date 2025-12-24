import fs from "fs-extra";
import path from "path";

export function createViteEnv(root: string) {
  fs.writeFileSync(
    path.join(root, "src/vite-env.d.ts"),
    `/// <reference types="vite/client" />

declare module "virtual:flick-routes" {
  const routes: Array<{
    path: string;
    component: () => Promise<{ default: () => Node }>;
  }>;
  export { routes };
}
`
  );
}
