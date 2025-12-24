import fs from "fs-extra";
import path from "path";

export function createFeatureList(root: string) {
  fs.writeFileSync(
    path.join(root, "src/components/FeatureList.tsx"),
    `export default function FeatureList() {
  const features = [
    { name: "signal()", description: "Fine-grained reactive state" },
    { name: "effect()", description: "Auto-tracking side effects" },
    { name: "resource()", description: "Async data fetching with Suspense" },
    { name: "lazy()", description: "Code-splitting and lazy loading" },
    { name: "Suspense", description: "Loading states for async content" },
    { name: "Router", description: "File-based routing with dynamic params" },
  ];

  return (
    <ul class="space-y-3">
      {features.map((feature) => (
        <li class="flex items-start gap-3">
          <code class="bg-gray-100 px-2 py-1 rounded text-sm font-mono text-blue-600">
            {feature.name}
          </code>
          <span class="text-gray-600">{feature.description}</span>
        </li>
      ))}
    </ul>
  );
}
`
  );
}
