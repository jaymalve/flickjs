import { Link } from 'react-router-dom';
import Navigation from '@/components/Navigation';
import Footer from '@/components/Footer';
import { Badge } from '@/components/ui/badge';
import { ScanRulesContent } from '@/components/ScanRulesContent';

const Scan = () => {
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
              <div className="border border-stone-800 rounded p-4 leading-relaxed">
                <pre>
                  <code>
                    <span className="text-stone-600">$</span>
                    <span className="text-stone-300"> npm install -g @flickjs/scan</span>
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
                  <span className="text-stone-600"># Generate a starter config</span>
                  {'\n'}
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> scan init</span>
                  {'\n\n'}
                  <span className="text-stone-600"># Scan your project</span>
                  {'\n'}
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> scan</span>
                  {'\n\n'}
                  <span className="text-stone-600"># Agent-friendly JSON output</span>
                  {'\n'}
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> scan --format agent-json</span>
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
