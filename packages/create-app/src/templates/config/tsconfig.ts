import fs from "fs-extra";
import path from "path";

export function createTsconfig(root: string) {
  fs.writeFileSync(
    path.join(root, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          target: "ES2020",
          module: "ESNext",
          moduleResolution: "bundler",
          jsx: "preserve",
          strict: true,
          esModuleInterop: true,
          skipLibCheck: true,
          types: ["bun-types"],
        },
        include: ["src/**/*"],
      },
      null,
      2
    ) + "\n"
  );
}
