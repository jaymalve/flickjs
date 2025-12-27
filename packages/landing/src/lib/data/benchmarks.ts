// Benchmark data - update these when you run new benchmarks

export const performanceData = [
  { benchmark: "Create 1,000 rows", flick: 58.1, solid: 52.5, react: 64.3 },
  { benchmark: "Replace 1,000 rows", flick: 65.6, solid: 61.0, react: 76.4 },
  { benchmark: "Partial update", flick: 30.7, solid: 30.9, react: 37.7 },
  { benchmark: "Select row", flick: 7.7, solid: 6.5, react: 8.2 },
  { benchmark: "Swap rows", flick: 233.2, solid: 36.3, react: 250.4 },
  { benchmark: "Remove row", flick: 27.8, solid: 26.1, react: 30.5 },
  { benchmark: "Create 10,000 rows", flick: 611.2, solid: 548.7, react: 680.3 },
  { benchmark: "Append rows", flick: 65.9, solid: 62.3, react: 78.1 },
  { benchmark: "Clear rows", flick: 34.4, solid: 28.9, react: 42.1 },
];

export const bundleSizeData = [
  { framework: "Flick", size: 1.9, color: "bg-emerald-500" },
  { framework: "SolidJS", size: 4.5, color: "bg-blue-500" },
  { framework: "React", size: 51.4, color: "bg-purple-500" },
];

export const memoryData = [
  { framework: "Flick", memory: 2.43, color: "bg-emerald-500" },
  { framework: "SolidJS", memory: 2.78, color: "bg-blue-500" },
  { framework: "React", memory: 4.61, color: "bg-purple-500" },
];

export const startupData = [
  { framework: "Flick", time: 91.8, color: "bg-emerald-500" },
  { framework: "SolidJS", time: 94.2, color: "bg-blue-500" },
  { framework: "React", time: 503.8, color: "bg-purple-500" },
];
