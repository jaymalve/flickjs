import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

type Tool =
  | {
      name: string;
      description: string;
      href: string;
    }
  | {
      name: string;
      description: string;
      comingSoon: true;
    };

const tools: Tool[] = [
  {
    name: '@flickjs/scan',
    description: 'Hyper-fast JavaScript scan tool with sub-second speeds',
    href: '/scan'
  },
  {
    name: '@flickjs/react',
    description: 'Surgical re-renders for React and Next.js.',
    comingSoon: true
  }
];

const Tools = () => {
  return (
    <section className="container pb-12 lg:pb-16">
      <div className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold tracking-tighter text-foreground">Tools</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {tools.map((tool) => {
            const isSoon = 'comingSoon' in tool && tool.comingSoon;

            const content = (
              <>
                <span
                  className={cn(
                    'text-base font-medium',
                    isSoon ? 'text-stone-500' : 'text-stone-300'
                  )}
                >
                  {tool.name}
                </span>
                <p
                  className={cn(
                    'text-sm leading-relaxed',
                    isSoon ? 'text-stone-600' : 'text-stone-500'
                  )}
                >
                  {tool.description}
                </p>
                {isSoon ? (
                  <Badge
                    variant="outline"
                    className="border-stone-700 text-stone-400 font-normal w-fit"
                  >
                    Coming soon
                  </Badge>
                ) : null}
              </>
            );

            if ('comingSoon' in tool && tool.comingSoon) {
              return (
                <div
                  key={tool.name}
                  className="bg-card/50 border border-stone-900 rounded p-6 flex flex-col gap-2 opacity-90"
                >
                  {content}
                </div>
              );
            }

            if ('href' in tool) {
              return (
                <a
                  key={tool.name}
                  href={tool.href}
                  className="bg-card border border-stone-800 rounded p-6 flex flex-col gap-2 transition-colors hover:border-stone-700"
                >
                  {content}
                </a>
              );
            }

            return null;
          })}
        </div>
      </div>
    </section>
  );
};

export default Tools;
