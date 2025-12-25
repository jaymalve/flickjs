import { transformSync } from "@babel/core";
import { createRequire } from "module";
import type { Plugin } from "vite";

const require = createRequire(import.meta.url);
const flickCompiler = require("@flickjs/compiler").default;

export default function flick(): Plugin {
  return {
    name: "vite-plugin-flick",
    enforce: "pre",
    transform(code, id) {
      if (!/\.[jt]sx$/.test(id)) return null;

      const result = transformSync(code, {
        filename: id,
        plugins: [flickCompiler],
        parserOpts: { plugins: ["jsx", "typescript"] },
        sourceMaps: true,
        cloneInputAst: false,
        configFile: false,
        babelrc: false,
      });

      if (!result?.code) return null;
      return { code: result.code, map: result.map };
    },
  };
}
