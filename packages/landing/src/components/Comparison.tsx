import { Check, X } from "lucide-react";

const comparisonData = [
  {
    feature: "Runtime Size",
    flick: "~300B",
    solid: "~7KB",
    vue: "~33KB",
    react: "~40KB",
  },
  {
    feature: "Virtual DOM",
    flick: false,
    solid: false,
    vue: true,
    react: true,
  },
  {
    feature: "Fine-grained Reactivity",
    flick: true,
    solid: true,
    vue: true,
    react: false,
  },
  {
    feature: "Zero Dependencies",
    flick: true,
    solid: true,
    vue: false,
    react: false,
  },
  {
    feature: "Template Literals",
    flick: true,
    solid: false,
    vue: false,
    react: false,
  },
];

const Comparison = () => {
  return (
    <section className="py-16 md:py-24 border-t border-input">
      <div className="container">
        {/* Section header */}
        <div className="text-center mb-12">
          <h2 className="text-2xl md:text-3xl font-bold tracking-tight text-foreground mb-4">
            How Flick compares
          </h2>
          <p className="text-muted-foreground max-w-lg mx-auto">
            See how Flick stacks up against popular frameworks.
          </p>
        </div>

        {/* Table */}
        <div className="max-w-4xl mx-auto overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-input">
                <th className="text-left py-4 px-4 font-medium text-muted-foreground">Feature</th>
                <th className="text-center py-4 px-4 font-semibold text-accent">Flick</th>
                <th className="text-center py-4 px-4 font-medium text-muted-foreground">SolidJS</th>
                <th className="text-center py-4 px-4 font-medium text-muted-foreground">Vue</th>
                <th className="text-center py-4 px-4 font-medium text-muted-foreground">React</th>
              </tr>
            </thead>
            <tbody>
              {comparisonData.map((row, index) => (
                <tr key={index} className="border-b border-input/50">
                  <td className="py-4 px-4 text-foreground">{row.feature}</td>
                  <td className="py-4 px-4 text-center">
                    <CellValue value={row.flick} highlight />
                  </td>
                  <td className="py-4 px-4 text-center">
                    <CellValue value={row.solid} />
                  </td>
                  <td className="py-4 px-4 text-center">
                    <CellValue value={row.vue} />
                  </td>
                  <td className="py-4 px-4 text-center">
                    <CellValue value={row.react} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
};

const CellValue = ({ value, highlight = false }: { value: string | boolean; highlight?: boolean }) => {
  if (typeof value === "boolean") {
    return value ? (
      <Check className={`h-4 w-4 mx-auto ${highlight ? "text-accent" : "text-muted-foreground"}`} />
    ) : (
      <X className="h-4 w-4 mx-auto text-muted-foreground/50" />
    );
  }
  return (
    <span className={highlight ? "text-accent font-semibold" : "text-muted-foreground"}>
      {value}
    </span>
  );
};

export default Comparison;
