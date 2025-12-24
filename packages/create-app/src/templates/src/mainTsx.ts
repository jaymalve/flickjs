import fs from "fs-extra";
import path from "path";

export function createMainTsx(root: string) {
  fs.writeFileSync(
    path.join(root, "src/main.tsx"),
    `import { mount } from "@flickjs/runtime";
import { Router } from "@flickjs/router";
import { routes } from "virtual:flick-routes";

// Mount the router to the app
mount(() => Router({ routes }), document.getElementById("app")!);
`
  );
}
