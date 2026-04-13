import Navigation from '@/components/Navigation';
import Footer from '@/components/Footer';

const Scan = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <main>
        <section className="container py-12 lg:py-16">
          <div className="flex flex-col gap-4">
            <h1 className="text-lg font-semibold tracking-tighter text-foreground">Flick Scan</h1>
            <p className="text-base text-stone-400 leading-relaxed">
              Rust-powered JavaScript and TypeScript linter with semantic rules and millisecond cold
              starts. Works with any project — React, Next.js, React Native, Express, and more.
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
              <p>
                Or with Cargo:{' '}
                <span className="text-stone-300">cargo install flick-scan</span>
              </p>
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
                  <span className="text-stone-300"> flick-scan init</span>
                  {'\n\n'}
                  <span className="text-stone-600"># Lint your project</span>
                  {'\n'}
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> flick-scan check .</span>
                  {'\n\n'}
                  <span className="text-stone-600"># Interactive TUI</span>
                  {'\n'}
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> flick-scan check . --format tui</span>
                  {'\n\n'}
                  <span className="text-stone-600"># JSON output for CI</span>
                  {'\n'}
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> flick-scan check . --format json</span>
                </code>
              </pre>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Configuration</h2>
            <p className="text-sm text-stone-500 leading-relaxed">
              Create a <span className="text-stone-300">flick.json</span> in your project root, or
              run <span className="text-stone-300">flick-scan init</span> to generate one
              automatically.
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
                        'react/no-fetch-in-effect': 'warn',
                      },
                    },
                    null,
                    2
                  )}
                </code>
              </pre>
            </div>
            <p className="text-sm text-stone-500 leading-relaxed">
              With <span className="text-stone-300">"detect": true</span>, Flick Scan auto-enables
              rules for React, Next.js, React Native, and server-side frameworks when detected from
              your <span className="text-stone-300">package.json</span>.
            </p>
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
                    'Written in Rust with OXC for parsing. No JIT warmup, no Node.js overhead.',
                },
                {
                  title: '70+ built-in rules',
                  description:
                    'Core JS/TS, React, Next.js, React Native, and server-side rules out of the box.',
                },
                {
                  title: 'Framework detection',
                  description:
                    'Reads your package.json and auto-enables matching rule categories. No manual config needed.',
                },
                {
                  title: 'Adaptive cache',
                  description:
                    'Smart caching that only activates when it beats a cold run. No stale results.',
                },
                {
                  title: 'Plain-English rules',
                  description:
                    'Write rules in natural language. They compile to native IR and run at full speed.',
                },
                {
                  title: 'Interactive TUI',
                  description:
                    'Browse results and rules in a terminal UI. Open files in your editor directly.',
                },
              ].map((feature) => (
                <div key={feature.title} className="flex flex-col gap-1">
                  <span className="text-stone-300 font-medium text-sm">{feature.title}</span>
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
              <p>
                macOS (arm64, x64) &middot; Linux (x64, arm64) &middot; Windows (x64)
              </p>
            </div>
          </div>
        </section>
      </main>
      <Footer />
    </div>
  );
};

export default Scan;
