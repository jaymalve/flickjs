const Installation = () => {
  return (
    <section className="container pb-12 lg:pb-16">
      <div className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold tracking-tighter text-foreground">
          Quick start
        </h2>
        <div className="flex flex-col gap-2 text-sm text-stone-500">
          <div className="border border-stone-800 rounded p-4 leading-relaxed">
            <pre>
              <code>
                <span className="text-stone-600">$</span>
                <span className="text-stone-300">
                  {" "}
                  bunx create-flick-app my-app
                </span>
                {"\n"}
                <span className="text-stone-600">$</span>
                <span className="text-stone-300"> cd my-app && bun dev</span>
              </code>
            </pre>
          </div>
          <p>
            Or just the linter:{" "}
            <span className="text-stone-300">cargo install flint</span>
          </p>
        </div>
      </div>
    </section>
  );
};

export default Installation;
