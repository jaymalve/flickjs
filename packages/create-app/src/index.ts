#!/usr/bin/env node
import fs from "fs-extra";
import path from "path";
import {
  createPackageJson,
  createGitignore,
  createViteConfig,
  createTailwindConfig,
  createTsconfig,
  createIndexHtml,
  createIndexCss,
  createMainTsx,
  createHomePage,
  createTodosPage,
  createTodoDetailPage,
  createAboutPage,
  createFeatureList,
} from "./templates";

const name = process.argv[2] || "flick-app";
const root = path.join(process.cwd(), name);

// Create directories
fs.ensureDirSync(root);
fs.ensureDirSync(path.join(root, "src"));
fs.ensureDirSync(path.join(root, "src/pages"));
fs.ensureDirSync(path.join(root, "src/pages/todos"));
fs.ensureDirSync(path.join(root, "src/components"));

// Create config files
createPackageJson(root, name);
createGitignore(root);
createViteConfig(root);
createTailwindConfig(root);
createTsconfig(root);
createIndexHtml(root);

// Create source files
createIndexCss(root);
createMainTsx(root);

// Create pages
createHomePage(root);
createTodosPage(root);
createTodoDetailPage(root);
createAboutPage(root);

// Create components
createFeatureList(root);

// Output success message
console.log("Updated Flick app created!");
console.log("");
console.log("Next steps:");
console.log(`  cd ${name}`);
console.log("  bun install");
console.log("  bun dev");
