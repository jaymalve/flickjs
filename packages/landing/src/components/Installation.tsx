import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";

const commands = [
  { label: "Create new app", command: "bunx create-flick-app my-app" },
  { label: "Navigate to app", command: "cd my-app" },
  { label: "Install dependencies", command: "bun install" },
  { label: "Start dev server", command: "bun dev" },
];

const Installation = () => {
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const handleCopy = async (command: string, index: number) => {
    await navigator.clipboard.writeText(command);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  return (
    <section id="docs" className="py-16 md:py-24 border-t border-input">
      <div className="container">
        <div className="max-w-2xl mx-auto">
          {/* Section header */}
          <div className="text-center mb-10">
            <h2 className="text-2xl md:text-3xl font-bold tracking-tight text-foreground mb-4">
              Quick Start
            </h2>
            <p className="text-muted-foreground">
              Get up and running in seconds.
            </p>
          </div>

          {/* Terminal */}
          <div className="code-block rounded overflow-hidden">
            {/* Terminal header */}
            <div className="flex items-center gap-2 px-4 py-3 border-b border-input">
              <div className="flex gap-1.5">
                <div className="w-3 h-3 rounded-full bg-muted-foreground/20" />
                <div className="w-3 h-3 rounded-full bg-muted-foreground/20" />
                <div className="w-3 h-3 rounded-full bg-muted-foreground/20" />
              </div>
              <span className="ml-2 text-xs text-muted-foreground font-mono">
                Terminal
              </span>
            </div>

            {/* Commands */}
            <div className="p-4 space-y-4">
              {commands.map((cmd, index) => (
                <div key={index} className="group">
                  <div className="text-xs text-muted-foreground mb-1">
                    # {cmd.label}
                  </div>
                  <div className="flex items-center justify-between gap-4 p-3 bg-background/50 rounded border border-input">
                    <div className="flex items-center gap-2 font-mono text-sm">
                      <span className="text-accent">$</span>
                      <span className="text-foreground">{cmd.command}</span>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
                      onClick={() => handleCopy(cmd.command, index)}
                    >
                      {copiedIndex === index ? (
                        <Check className="h-3.5 w-3.5 text-accent" />
                      ) : (
                        <Copy className="h-3.5 w-3.5 text-muted-foreground" />
                      )}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default Installation;
