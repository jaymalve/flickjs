const tools = [
  {
    name: '@flickjs/scan',
    description: 'Hyper-fast JavaScript scan tool with sub-second speeds',
    href: '/scan'
  },
  {
    name: '@flickjs/react',
    description: 'Surgical re-renders for React and Next.js.',
    href: '/react'
  }
];

const Tools = () => {
  return (
    <section className="container pb-12 lg:pb-16">
      <div className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold tracking-tighter text-foreground">Tools</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {tools.map((tool) => (
            <a
              key={tool.name}
              href={tool.href}
              className="bg-card border border-stone-800 rounded p-6 flex flex-col gap-2 transition-colors hover:border-stone-700"
            >
              <span className="text-base font-medium text-stone-300">{tool.name}</span>
              <p className="text-sm text-stone-500">{tool.description}</p>
            </a>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Tools;
