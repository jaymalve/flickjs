const Hero = () => {
  return (
    <section className="container py-12 lg:py-16">
      <div className="flex flex-col gap-4">
        <h1 className="text-lg font-semibold tracking-tighter text-foreground">
          Fast JavaScript tools.
        </h1>
        <p className="text-base text-stone-400 leading-relaxed">
          FlickJS is a growing family of high-performance tools for JavaScript and TypeScript. Each
          tool works standalone — use what you need, skip what you don't.
        </p>
        <p className="text-base text-stone-400 leading-relaxed">
          A{' '}
          <a href="/docs/flint" className="link text-stone-300">
            linter
          </a>{' '}
          built in Rust with millisecond cold starts. A{' '}
          <a href="/docs/compiler" className="link text-stone-300">
            compiler
          </a>{' '}
          that turns JSX into vanilla JS at build time. A{' '}
          <a href="/docs/runtime" className="link text-stone-300">
            runtime
          </a>{' '}
          that ships ~300 bytes of reactive UI. A{' '}
          <a href="/docs/router" className="link text-stone-300">
            router
          </a>{' '}
          with file-based routing. And an{' '}
          <a href="/docs/ai" className="link text-stone-300">
            AI SDK
          </a>{' '}
          with reactive bindings.
        </p>
      </div>
    </section>
  );
};

export default Hero;
