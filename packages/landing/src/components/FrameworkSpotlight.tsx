const FrameworkSpotlight = () => {
  return (
    <section className="container pb-12 lg:pb-16">
      <div className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold tracking-tighter text-foreground">Why Flick?</h2>
        <p className="text-base text-stone-400 leading-relaxed">
          Fine-grained reactivity with direct DOM manipulation. No diffing, no reconciliation — just
          surgical updates to exactly what changed. Your bundle stays tiny. Your app stays fast.
        </p>
        <div className="text-sm text-stone-500 leading-relaxed flex flex-col gap-2">
          <p>
            <span className="text-stone-300">~300 bytes</span> runtime.{' '}
            <span className="text-stone-300">0</span> dependencies.{' '}
            <span className="text-stone-300">0</span> Virtual DOM.
          </p>
        </div>
        <div className="mt-2 border border-stone-800 rounded p-4 text-sm leading-relaxed">
          <pre className="text-stone-500">
            <code>
              <span className="text-stone-300">import</span>
              {' { fx, run } '}
              <span className="text-stone-300">from</span>
              {" '@flickjs/runtime';\n\n"}
              <span className="text-stone-300">const</span>
              {' count = '}
              <span className="text-stone-400">fx</span>
              {'(0);\n'}
              <span className="text-stone-300">const</span>
              {' doubled = () => count() * 2;\n\n'}
              <span className="text-stone-400">run</span>
              {'(() => console.'}
              <span className="text-stone-400">log</span>
              {"('Count:', count()));"}
            </code>
          </pre>
        </div>
        <p className="text-sm text-stone-500">
          <a href="/docs/getting-started" className="link text-stone-400">
            Get started with the framework →
          </a>
        </p>
      </div>
    </section>
  );
};

export default FrameworkSpotlight;
