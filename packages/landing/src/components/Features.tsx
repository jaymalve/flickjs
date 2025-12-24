import { Zap, Box, Feather, Package } from "lucide-react";

const features = [
  {
    icon: Zap,
    title: "Fine-grained Reactivity",
    description: "Updates only what changed. No diffing, no reconciliation—just surgical DOM updates.",
  },
  {
    icon: Box,
    title: "No Virtual DOM",
    description: "Direct DOM manipulation for maximum performance. Skip the abstraction overhead.",
  },
  {
    icon: Feather,
    title: "Featherweight",
    description: "A runtime so small, your bundle will barely notice it's there.",
  },
  {
    icon: Package,
    title: "Zero Dependencies",
    description: "No framework overhead in production. Just pure, lean JavaScript.",
  },
];

const Features = () => {
  return (
    <section className="py-16 md:py-24 border-t border-input">
      <div className="container">
        {/* Section header */}
        <div className="text-center mb-12 md:mb-16">
          <h2 className="text-2xl md:text-3xl font-bold tracking-tight text-foreground mb-4">
            Built for performance
          </h2>
          <p className="text-muted-foreground max-w-lg mx-auto">
            Every byte counts. Every millisecond matters. Flick is engineered from the ground up for speed.
          </p>
        </div>

        {/* Bento grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-4xl mx-auto">
          {features.map((feature, index) => (
            <div
              key={index}
              className="card-glow bg-card p-6 md:p-8 rounded"
            >
              <feature.icon className="h-5 w-5 text-accent mb-4" />
              <h3 className="text-lg font-semibold text-foreground mb-2 tracking-tight">
                {feature.title}
              </h3>
              <p className="text-sm text-muted-foreground leading-relaxed">
                {feature.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
