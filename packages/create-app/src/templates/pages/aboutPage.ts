import fs from "fs-extra";
import path from "path";

export function createAboutPage(root: string) {
  fs.writeFileSync(
    path.join(root, "src/pages/about.tsx"),
    `import { lazy, Suspense } from "@flickjs/runtime";
import { Link } from "@flickjs/router";

// Lazy loaded component - loaded only when rendered
const LazyFeatureList = lazy(() => import("../components/FeatureList"));

export default function About() {
  return (
    <div class="min-h-screen bg-gray-100 p-8">
      <div class="max-w-4xl mx-auto">
        <div class="mb-6">
          <Link href="/" class="text-blue-500 hover:underline">
            ← Back to Home
          </Link>
        </div>

        <h1 class="text-4xl font-bold text-gray-800 mb-6">About Flick</h1>

        <div class="bg-white rounded-lg shadow-md p-6 mb-6">
          <h2 class="text-2xl font-semibold mb-4">What is Flick?</h2>
          <p class="text-gray-600 mb-4">
            Flick is a lightweight, reactive JavaScript framework with
            fine-grained reactivity. It features a tiny runtime, compiled JSX,
            and intuitive APIs for building modern web applications.
          </p>
        </div>

        <div class="bg-white rounded-lg shadow-md p-6">
          <h2 class="text-2xl font-semibold mb-4">
            Features (Lazy Loaded Component)
          </h2>
          <Suspense
            fallback={<div class="text-gray-500">Loading features...</div>}
          >
            <LazyFeatureList />
          </Suspense>
        </div>
      </div>
    </div>
  );
}
`
  );
}
