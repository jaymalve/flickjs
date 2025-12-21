import type { Plugin } from "vite";
import { readdir, stat } from "fs/promises";
import { existsSync } from "fs";
import { join, relative, resolve } from "path";
import { filePathToRoute, type Route } from "./utils";

const VIRTUAL_MODULE_ID = "virtual:flick-routes";
const RESOLVED_VIRTUAL_MODULE_ID = "\0" + VIRTUAL_MODULE_ID;

interface RouteFile {
  path: string;
  importPath: string;
  route: string;
}

/**
 * Recursively scan directory for route files
 */
async function scanPagesDirectory(
  dir: string,
  baseDir: string,
  pagesDir: string = "pages"
): Promise<RouteFile[]> {
  const routes: RouteFile[] = [];
  const entries = await readdir(dir);

  for (const entry of entries) {
    const fullPath = join(dir, entry);
    const stats = await stat(fullPath);

    if (stats.isDirectory()) {
      // Recursively scan subdirectories
      const subRoutes = await scanPagesDirectory(fullPath, baseDir, pagesDir);
      routes.push(...subRoutes);
    } else if (/\.(tsx?|jsx?)$/.test(entry)) {
      // Found a route file
      const relativePath = relative(baseDir, fullPath);
      const route = filePathToRoute(relativePath, pagesDir);
      // Use absolute path for virtual module imports
      const importPath = fullPath.replace(/\\/g, "/");

      routes.push({
        path: fullPath,
        importPath,
        route,
      });
    }
  }

  return routes;
}

/**
 * Generate route manifest code
 */
function generateRouteManifest(routes: RouteFile[]): string {
  const routeImports = routes
    .map(
      (r, i) =>
        `  { path: ${JSON.stringify(
          r.route
        )}, component: () => import(${JSON.stringify(r.importPath)}) }`
    )
    .join(",\n");

  return `export const routes = [\n${routeImports}\n];\n`;
}

export function flickRouter(options?: {
  pagesDir?: string;
  root?: string;
}): Plugin {
  const pagesDir = options?.pagesDir || "pages";
  const root = options?.root || process.cwd();

  return {
    name: "flick-router",
    resolveId(id) {
      if (id === VIRTUAL_MODULE_ID) {
        return RESOLVED_VIRTUAL_MODULE_ID;
      }
    },
    async load(id) {
      if (id === RESOLVED_VIRTUAL_MODULE_ID) {
        // Find src directory (common in Vite projects)
        const srcDir = resolve(root, "src");
        const pagesPath = join(srcDir, pagesDir);

        // Check if pages directory exists
        if (!existsSync(pagesPath)) {
          console.warn(
            `[flick-router] Pages directory not found at ${pagesPath}. Make sure you have a ${pagesDir}/ directory in your src folder.`
          );
          return `export const routes = [];\n`;
        }

        try {
          const routeFiles = await scanPagesDirectory(
            pagesPath,
            srcDir,
            pagesDir
          );

          if (routeFiles.length === 0) {
            console.warn(
              `[flick-router] No route files found in ${pagesPath}. Make sure you have .tsx or .jsx files in your ${pagesDir}/ directory.`
            );
            return `export const routes = [];\n`;
          }

          // Sort routes: exact matches first, then dynamic, then catch-all
          routeFiles.sort((a, b) => {
            const aHasParams = a.route.includes(":");
            const bHasParams = b.route.includes(":");
            const aHasCatchAll = a.route.includes("*");
            const bHasCatchAll = b.route.includes("*");

            if (aHasCatchAll && !bHasCatchAll) return 1;
            if (!aHasCatchAll && bHasCatchAll) return -1;
            if (aHasParams && !bHasParams) return 1;
            if (!aHasParams && bHasParams) return -1;
            return 0;
          });

          return generateRouteManifest(routeFiles);
        } catch (error: any) {
          // Handle case where directory doesn't exist or can't be read
          if (error.code === "ENOENT") {
            console.warn(
              `[flick-router] Pages directory not found at ${pagesPath}.`
            );
          } else {
            console.error(
              `[flick-router] Error scanning pages directory:`,
              error
            );
          }
          return `export const routes = [];\n`;
        }
      }
    },
    configureServer(server) {
      // Watch for file changes in pages directory
      const pagesPath = resolve(root, "src", pagesDir);
      if (existsSync(pagesPath)) {
        server.watcher.add(pagesPath);
      }
    },
  };
}
