import Navigation from "@/components/Navigation";
import Footer from "@/components/Footer";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

// Benchmark data - update these when you run new benchmarks
const performanceData = [
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

const bundleSizeData = [
  { framework: "Flick", size: 1.9, color: "bg-emerald-500" },
  { framework: "SolidJS", size: 4.5, color: "bg-blue-500" },
  { framework: "React", size: 51.4, color: "bg-purple-500" },
];

const memoryData = [
  { framework: "Flick", memory: 2.43, color: "bg-emerald-500" },
  { framework: "SolidJS", memory: 2.78, color: "bg-blue-500" },
  { framework: "React", memory: 4.61, color: "bg-purple-500" },
];

const startupData = [
  { framework: "Flick", time: 91.8, color: "bg-emerald-500" },
  { framework: "SolidJS", time: 94.2, color: "bg-blue-500" },
  { framework: "React", time: 503.8, color: "bg-purple-500" },
];

function getWinner(row: { flick: number; solid: number; react: number }) {
  const min = Math.min(row.flick, row.solid, row.react);
  if (row.flick === min) return "flick";
  if (row.solid === min) return "solid";
  return "react";
}

function getCompetitiveStatus(row: { flick: number; solid: number; react: number }) {
  const flickVsSolid = ((row.flick - row.solid) / row.solid) * 100;
  const flickVsReact = ((row.react - row.flick) / row.flick) * 100;

  // Flick is within 20% of Solid and faster than React (>5%)
  const isCompetitive = flickVsSolid <= 20 && flickVsSolid > 0 && flickVsReact > 5;

  return { isCompetitive };
}

function BarChart({
  data,
  valueKey,
  unit,
  maxValue,
}: {
  data: { framework: string; color: string; [key: string]: number | string }[];
  valueKey: string;
  unit: string;
  maxValue?: number;
}) {
  const max = maxValue || Math.max(...data.map((d) => d[valueKey] as number));

  return (
    <div className="space-y-3">
      {data.map((item) => {
        const value = item[valueKey] as number;
        const width = (value / max) * 100;
        return (
          <div key={item.framework} className="space-y-1">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">{item.framework}</span>
              <span className="font-mono font-medium">
                {value} {unit}
              </span>
            </div>
            <div className="h-3 bg-muted rounded-full overflow-hidden">
              <div
                className={`h-full ${item.color} rounded-full transition-all duration-500`}
                style={{ width: `${width}%` }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}

const Benchmarks = () => {
  return (
    <div className="min-h-screen bg-background overflow-x-hidden">
      <Navigation />
      <main className="pt-20 pb-16">
        <div className="container max-w-5xl">
          {/* Header */}
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold mb-4">Benchmarks</h1>
            <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
              Official js-framework-benchmark results comparing Flick against
              SolidJS and React. Lower is better for all metrics.
            </p>
            <p className="text-sm text-muted-foreground mt-2">
              Last updated: December 2024
            </p>
          </div>

          {/* Key Metrics Cards */}
          <div className="grid md:grid-cols-3 gap-6 mb-12">
            <div className="bg-card border border-border rounded-lg p-6">
              <div className="text-sm text-muted-foreground mb-1">
                Bundle Size (gzip)
              </div>
              <div className="text-3xl font-bold text-emerald-500">1.9 KB</div>
              <div className="text-sm text-muted-foreground mt-1">
                vs React 51.4 KB (96% smaller)
              </div>
            </div>
            <div className="bg-card border border-border rounded-lg p-6">
              <div className="text-sm text-muted-foreground mb-1">
                Memory Usage
              </div>
              <div className="text-3xl font-bold text-emerald-500">2.43 MB</div>
              <div className="text-sm text-muted-foreground mt-1">
                vs React 4.61 MB (47% less)
              </div>
            </div>
            <div className="bg-card border border-border rounded-lg p-6">
              <div className="text-sm text-muted-foreground mb-1">
                First Paint
              </div>
              <div className="text-3xl font-bold text-emerald-500">91.8 ms</div>
              <div className="text-sm text-muted-foreground mt-1">
                vs React 503.8 ms (5x faster)
              </div>
            </div>
          </div>

          {/* Bundle Size Chart */}
          <div className="mb-12">
            <h2 className="text-2xl font-semibold mb-4">
              Bundle Size Comparison
            </h2>
            <div className="bg-card border border-border rounded-lg p-6">
              <BarChart
                data={bundleSizeData}
                valueKey="size"
                unit="KB"
                maxValue={60}
              />
            </div>
          </div>

          {/* Memory Chart */}
          <div className="mb-12">
            <h2 className="text-2xl font-semibold mb-4">Memory Usage</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Heap memory after creating 1,000 rows
            </p>
            <div className="bg-card border border-border rounded-lg p-6">
              <BarChart data={memoryData} valueKey="memory" unit="MB" />
            </div>
          </div>

          {/* Startup Time Chart */}
          <div className="mb-12">
            <h2 className="text-2xl font-semibold mb-4">First Paint</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Time to first meaningful paint
            </p>
            <div className="bg-card border border-border rounded-lg p-6">
              <BarChart data={startupData} valueKey="time" unit="ms" />
            </div>
          </div>

          {/* Performance Table */}
          <div className="mb-12">
            <h2 className="text-2xl font-semibold mb-4">
              Runtime Performance
            </h2>
            <p className="text-muted-foreground text-sm mb-4">
              Median time in milliseconds (15 runs each). Lower is better.
            </p>
            <div className="bg-card border border-border rounded-lg overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[200px]">Benchmark</TableHead>
                    <TableHead className="text-right">FlickJS</TableHead>
                    <TableHead className="text-right">SolidJS</TableHead>
                    <TableHead className="text-right">React</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {performanceData.map((row) => {
                    const winner = getWinner(row);
                    const { isCompetitive } = getCompetitiveStatus(row);
                    return (
                      <TableRow key={row.benchmark}>
                        <TableCell className="font-medium">
                          {row.benchmark}
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          <span
                            className={
                              winner === "flick"
                                ? "text-emerald-500 font-semibold"
                                : ""
                            }
                          >
                            {row.flick} ms
                          </span>
                          {winner === "flick" && (
                            <Badge
                              variant="outline"
                              className="ml-2 text-emerald-500 border-emerald-500/30"
                            >
                              Best
                            </Badge>
                          )}
                          {winner !== "flick" && isCompetitive && (
                            <Badge
                              variant="outline"
                              className="ml-2 text-amber-500 border-amber-500/30"
                            >
                              Near best
                            </Badge>
                          )}
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          <span
                            className={
                              winner === "solid"
                                ? "text-blue-500 font-semibold"
                                : ""
                            }
                          >
                            {row.solid} ms
                          </span>
                          {winner === "solid" && (
                            <Badge
                              variant="outline"
                              className="ml-2 text-blue-500 border-blue-500/30"
                            >
                              Best
                            </Badge>
                          )}
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          <span
                            className={
                              winner === "react"
                                ? "text-purple-500 font-semibold"
                                : ""
                            }
                          >
                            {row.react} ms
                          </span>
                          {winner === "react" && (
                            <Badge
                              variant="outline"
                              className="ml-2 text-purple-500 border-purple-500/30"
                            >
                              Best
                            </Badge>
                          )}
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </div>
          </div>

          {/* Notes */}
          <div className="bg-muted/50 border border-border rounded-lg p-6">
            <h3 className="font-semibold mb-3">About these benchmarks</h3>
            <ul className="text-sm text-muted-foreground space-y-2">
              <li>
                Results from the official{" "}
                <a
                  href="https://github.com/krausest/js-framework-benchmark"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-foreground underline underline-offset-2 hover:text-emerald-500"
                >
                  js-framework-benchmark
                </a>
              </li>
              <li>
                Tested on: macOS, Chrome, Node.js 24, 15 warm-up runs per test
              </li>
              <li>
                Flick v0.0.1-beta.3, SolidJS v1.9.3, React v19.2.0 (hooks)
              </li>
              <li>
                Note: Swap rows performance is an area of active optimization
              </li>
            </ul>
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
};

export default Benchmarks;
