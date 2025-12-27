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
import { CompareFramework, frameworkConfig } from "@/lib/frameworks";
import {
  performanceData,
  bundleSizeData,
  memoryData,
  startupData,
} from "@/lib/data/benchmarks";

interface BenchmarksProps {
  compareFrameworks?: CompareFramework[];
}

function getWinner(
  row: { flick: number; solid: number; react: number },
  frameworks: CompareFramework[]
) {
  const values: { key: string; value: number }[] = [
    { key: "flick", value: row.flick },
    { key: "react", value: row.react },
  ];

  // Add other frameworks if included
  if (frameworks.includes(CompareFramework.SolidJS)) {
    values.push({ key: "solid", value: row.solid });
  }

  const min = Math.min(...values.map((v) => v.value));
  return values.find((v) => v.value === min)?.key || "flick";
}

function getCompetitiveStatus(
  row: { flick: number; solid: number; react: number },
  frameworks: CompareFramework[]
) {
  // Only show "Near best" when comparing against SolidJS
  if (!frameworks.includes(CompareFramework.SolidJS)) {
    return { isCompetitive: false };
  }

  const flickVsSolid = ((row.flick - row.solid) / row.solid) * 100;
  const flickVsReact = ((row.react - row.flick) / row.flick) * 100;

  // Flick is within 20% of Solid and faster than React (>5%)
  const isCompetitive =
    flickVsSolid <= 20 && flickVsSolid > 0 && flickVsReact > 5;

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

const Benchmarks = ({
  compareFrameworks = [CompareFramework.SolidJS],
}: BenchmarksProps) => {
  const showSolid = compareFrameworks.includes(CompareFramework.SolidJS);

  // Filter bar chart data based on compareFrameworks
  const filterChartData = <T extends { framework: string }>(data: T[]) =>
    data.filter(
      (d) =>
        d.framework === "Flick" ||
        d.framework === "React" ||
        (d.framework === "SolidJS" && showSolid)
    );

  // Generate comparison text for header
  const comparisonText =
    compareFrameworks.length > 0
      ? `comparing Flick against ${compareFrameworks
          .map((f) => frameworkConfig[f].name)
          .join(", ")} and React`
      : "comparing Flick against React";

  return (
    <div className="min-h-screen bg-background overflow-x-hidden">
      <Navigation />
      <main className="pt-20 pb-16">
        <div className="container max-w-5xl">
          {/* Header */}
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold mb-4">Benchmarks</h1>
            <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
              Official{" "}
              <a
                href="https://github.com/krausest/js-framework-benchmark"
                target="_blank"
                rel="noopener noreferrer"
                className="text-foreground underline underline-offset-2 hover:text-emerald-500"
              >
                js-framework-benchmark
              </a>{" "}
              results {comparisonText}.<br /> Lower is better for all metrics.
            </p>
            <p className="text-sm text-muted-foreground mt-4">
              Last updated: 27th December, 2025
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
                data={filterChartData(bundleSizeData)}
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
              <BarChart
                data={filterChartData(memoryData)}
                valueKey="memory"
                unit="MB"
              />
            </div>
          </div>

          {/* Startup Time Chart */}
          <div className="mb-12">
            <h2 className="text-2xl font-semibold mb-4">First Paint</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Time to first meaningful paint
            </p>
            <div className="bg-card border border-border rounded-lg p-6">
              <BarChart
                data={filterChartData(startupData)}
                valueKey="time"
                unit="ms"
              />
            </div>
          </div>

          {/* Performance Table */}
          <div className="mb-12">
            <h2 className="text-2xl font-semibold mb-4">Runtime Performance</h2>
            <p className="text-muted-foreground text-sm mb-4">
              Median time in milliseconds (15 runs each).
              <br />
              Lower is better.
            </p>
            <div className="bg-card border border-border rounded-lg overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[200px]">Benchmark</TableHead>
                    <TableHead className="text-right">FlickJS</TableHead>
                    {showSolid && (
                      <TableHead className="text-right">SolidJS</TableHead>
                    )}
                    <TableHead className="text-right">React</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {performanceData.map((row) => {
                    const winner = getWinner(row, compareFrameworks);
                    const { isCompetitive } = getCompetitiveStatus(
                      row,
                      compareFrameworks
                    );
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
                          {winner === "flick" &&
                            compareFrameworks.length > 0 && (
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
                        {showSolid && (
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
                        )}
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
                Flick v0.0.1-beta.3
                {showSolid && ", SolidJS v1.9.3"}, React v19.2.0 (hooks)
              </li>
              <li>
                Note: I have been working on optimizations for the swap rows
                benchmark by optimizing the renderList algorithm that I am
                writing for Flick Compiler.
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
