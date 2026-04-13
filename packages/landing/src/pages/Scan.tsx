import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';
import { Check, Copy } from 'lucide-react';
import Navigation from '@/components/Navigation';
import Footer from '@/components/Footer';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScanRulesContent } from '@/components/ScanRulesContent';
import { cn } from '@/lib/utils';

const INSTALL_COMMAND = 'npm install -g @flickjs/scan';

const USAGE_COMMANDS: { comment: string; cmd: string }[] = [
  { comment: '# Generate a starter config', cmd: 'scan init' },
  { comment: '# Scan your project', cmd: 'scan' },
  { comment: '# Agent-friendly JSON output', cmd: 'scan --format agent-json' }
];

const Scan = () => {
  const [copied, setCopied] = useState(false);
  const [copiedUsageCmd, setCopiedUsageCmd] = useState<string | null>(null);
  const usageCopyResetRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const copyInstallCommand = async () => {
    try {
      await navigator.clipboard.writeText(INSTALL_COMMAND);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard may be unavailable (e.g. insecure context)
    }
  };

  const copyUsageCommand = async (cmd: string) => {
    try {
      await navigator.clipboard.writeText(cmd);
      setCopiedUsageCmd(cmd);
      if (usageCopyResetRef.current) clearTimeout(usageCopyResetRef.current);
      usageCopyResetRef.current = setTimeout(() => {
        setCopiedUsageCmd(null);
        usageCopyResetRef.current = null;
      }, 2000);
    } catch {
      // Clipboard unavailable
    }
  };

  useEffect(() => {
    return () => {
      if (usageCopyResetRef.current) clearTimeout(usageCopyResetRef.current);
    };
  }, []);

  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <main>
        <section className="container py-12 lg:py-16">
          <div className="flex flex-col gap-4">
            <h1 className="text-lg font-semibold tracking-tighter text-foreground">Flick Scan</h1>
            <p className="text-base text-stone-400 leading-relaxed">
              Catch anti-patterns and code smells at a sub-second speed.
            </p>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Install</h2>
            <div className="flex flex-col gap-2 text-sm text-stone-500">
              <div className="relative border border-stone-800 rounded p-4 pr-14 leading-relaxed">
                <Button
                  type="button"
                  variant="copy"
                  size="icon"
                  className="absolute top-2 right-2 h-8 w-8 border-stone-700 text-stone-400 hover:text-stone-200"
                  onClick={copyInstallCommand}
                  aria-label={copied ? 'Copied' : 'Copy install command'}
                >
                  {copied ? <Check className="text-emerald-400" /> : <Copy />}
                </Button>
                <pre>
                  <code>
                    <span className="text-stone-600">$</span>
                    <span className="text-stone-300"> {INSTALL_COMMAND}</span>
                  </code>
                </pre>
              </div>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Usage</h2>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  {USAGE_COMMANDS.map((item, index) => (
                    <span key={item.cmd}>
                      {index > 0 ? '\n\n' : null}
                      <span className="text-stone-600">{item.comment}</span>
                      {'\n'}
                      <span className="text-stone-600">$</span>
                      <span
                        role="button"
                        tabIndex={0}
                        title="Click to copy command"
                        className={cn(
                          'inline-flex max-w-full cursor-pointer flex-wrap items-baseline gap-2 rounded px-0.5 -mx-0.5 text-left text-stone-300 outline-none transition-colors',
                          'hover:text-stone-100 hover:underline decoration-stone-600 underline-offset-2',
                          'focus-visible:ring-1 focus-visible:ring-stone-500'
                        )}
                        onClick={() => copyUsageCommand(item.cmd)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            copyUsageCommand(item.cmd);
                          }
                        }}
                      >
                        {' '}
                        {item.cmd}
                        {copiedUsageCmd === item.cmd ? (
                          <span className="font-sans text-xs font-normal text-emerald-500/90 no-underline">
                            Copied
                          </span>
                        ) : null}
                      </span>
                    </span>
                  ))}
                </code>
              </pre>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">
              Configuration
            </h2>
            <p className="text-sm text-stone-500 leading-relaxed">
              Run <span className="text-stone-300">scan init</span> to generate a{' '}
              <span className="text-stone-300">flick.json</span> in your project root.
            </p>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code className="text-stone-300">
                  {JSON.stringify(
                    {
                      $schema: 'https://www.flickjs.com/scan/schema.json',
                      detect: true,
                      rules: {
                        'no-explicit-any': 'warn',
                        'no-unused-vars': 'error',
                        'no-console': 'warn',
                        'react/no-fetch-in-effect': 'warn'
                      }
                    },
                    null,
                    2
                  )}
                </code>
              </pre>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-baseline justify-between gap-2">
              <h2 className="text-lg font-semibold tracking-tighter text-foreground">Rules</h2>
              <Link
                to="/scan/rules"
                className="text-sm text-stone-500 hover:text-stone-300 transition-colors shrink-0"
              >
                Open standalone page
              </Link>
            </div>
            <ScanRulesContent />
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Features</h2>
            <div className="flex flex-col gap-4">
              {[
                {
                  title: 'Millisecond cold starts',
                  description:
                    'Written in Rust with OXC for parsing. No JIT warmup, no Node.js overhead.'
                },
                {
                  title: '90+ built-in rules',
                  description:
                    'Core JS/TS, React, Next.js, React Native, and server-side rules out of the box.'
                },
                {
                  title: 'Framework detection',
                  description:
                    'Reads your package.json and auto-enables matching rule categories. No manual config needed.'
                },
                {
                  title: 'Adaptive cache',
                  description:
                    'Smart caching that only activates when it beats a cold run. No stale results.'
                },
                {
                  title: 'Interactive TUI',
                  description:
                    'Browse results and rules in a terminal UI. Open files in your editor directly.'
                },
                {
                  title: 'Plain-English rules',
                  description:
                    'Write rules in natural language. They compile to native IR and run at full speed.',
                  comingSoon: true
                }
              ].map((feature) => (
                <div key={feature.title} className="flex flex-col gap-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-stone-300 font-medium text-sm">{feature.title}</span>
                    {feature.comingSoon ? (
                      <Badge
                        variant="outline"
                        className="border-stone-700 text-stone-400 font-normal"
                      >
                        Coming soon
                      </Badge>
                    ) : null}
                  </div>
                  <p className="text-sm text-stone-500 leading-relaxed">{feature.description}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Platforms</h2>
            <div className="text-sm text-stone-500 leading-relaxed">
              <p>macOS (arm64, x64) &middot; Linux (x64, arm64) &middot; Windows (x64)</p>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </div>
  );
};

export default Scan;
