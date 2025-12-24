import fs from "fs-extra";
import path from "path";

export function createIndexHtml(root: string) {
  fs.writeFileSync(
    path.join(root, "index.html"),
    `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Flick App</title>
    <link rel="stylesheet" href="/src/index.css">
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
`
  );
}
