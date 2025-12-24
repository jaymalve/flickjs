import fs from "fs-extra";
import path from "path";

export function createTailwindConfig(root: string) {
  fs.writeFileSync(
    path.join(root, "tailwind.config.js"),
    `/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {},
  },
  plugins: [],
};
`
  );
}
