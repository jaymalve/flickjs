import { transformSync } from "@babel/core";
import { createRequire } from "module";
import { readFileSync, statSync } from "fs";
import { resolve } from "path";
import { gzipSync } from "zlib";
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
    writeBundle(options, bundle) {
      const outputDir = options.dir || resolve(process.cwd(), "dist");
      let totalSize = 0;
      let totalGzipSize = 0;
      let assetCount = 0;

      // Calculate sizes for all assets
      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (chunk.type === "asset" || chunk.type === "chunk") {
          const filePath = resolve(outputDir, fileName);
          try {
            const stats = statSync(filePath);
            const size = stats.size;
            totalSize += size;
            assetCount++;

            // Calculate gzip size for compressible assets
            if (
              chunk.type === "chunk" ||
              fileName.endsWith(".js") ||
              fileName.endsWith(".css")
            ) {
              const content = readFileSync(filePath);
              const gzipped = gzipSync(content);
              totalGzipSize += gzipped.length;
            } else {
              // For other assets, use raw size as approximation
              totalGzipSize += size;
            }
          } catch (e) {
            // File might not exist yet, skip
          }
        }
      }

      // Format sizes
      const formatSize = (bytes: number) => {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} kB`;
        return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
      };

      // Print summary
      console.log("");
      console.log("📦 Bundle Size Summary:");
      console.log(`   Total assets: ${assetCount} files`);
      console.log(
        `   Total size: ${formatSize(totalSize)} │ gzip: ${formatSize(
          totalGzipSize
        )}`
      );
      console.log("");
    },
  };
}
