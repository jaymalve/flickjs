const tools = [
  {
    name: "Flint",
    description:
      "Rust-powered JS/TS linting with semantic rules and millisecond cold starts. Works with any project.",
    href: "/docs/flint",
  },
  {
    name: "Compiler",
    description:
      "Compiles JSX to vanilla JavaScript at build time. Zero runtime cost.",
    href: "/docs/compiler",
  },
  {
    name: "Runtime",
    description:
      "~300 byte reactive UI framework. Fine-grained updates, no Virtual DOM, no diffing.",
    href: "/docs/runtime",
  },
  {
    name: "Router",
    description: "File-based routing with dynamic params. Plug and play.",
    href: "/docs/router",
  },
  {
    name: "AI",
    description:
      "LLM integration with reactive bindings. Chat, agents, structured output.",
    href: "/docs/ai",
  },
];

const Tools = () => {
  return (
    <section className="container pb-12 lg:pb-16">
      <div className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold tracking-tighter text-foreground">
          The toolkit
        </h2>
        <div className="flex flex-col gap-4">
          {tools.map((tool) => (
            <div key={tool.name} className="flex flex-col gap-1">
              <a
                href={tool.href}
                className="link text-stone-300 font-medium text-base w-fit"
              >
                {tool.name}
              </a>
              <p className="text-sm text-stone-500 leading-relaxed">
                {tool.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Tools;
