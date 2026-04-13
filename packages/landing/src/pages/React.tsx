import Navigation from '@/components/Navigation';
import Footer from '@/components/Footer';

const ReactPage = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <main>
        <section className="container py-12 lg:py-16">
          <div className="flex flex-col gap-4">
            <h1 className="text-lg font-semibold tracking-tighter text-foreground">
              @flickjs/react
            </h1>
            <p className="text-base text-stone-400 leading-relaxed">
              Signals and AI hooks for React and Next.js. Surgical re-renders, zero dependency
              arrays.
            </p>
            <a
              href="/react/demo"
              className="border border-stone-800 rounded px-4 py-2 text-sm font-medium text-stone-300 w-fit transition-colors hover:border-stone-700 hover:text-foreground"
            >
              Live demo →
            </a>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Install</h2>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  <span className="text-stone-600">$</span>
                  <span className="text-stone-300"> npm install @flickjs/react</span>
                </code>
              </pre>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">
              Reactive hooks
            </h2>
            <p className="text-sm text-stone-500 leading-relaxed">
              Subscribe to <span className="text-stone-300">fx</span> signals inside React
              components. Only the components that read a signal re-render when it changes.
            </p>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  <span className="text-stone-600">{'// useFx — subscribe to a signal'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'import { fx, useFx } from \'@flickjs/react\''}</span>
                  {'\n\n'}
                  <span className="text-stone-300">{'const count = fx(0)'}</span>
                  {'\n\n'}
                  <span className="text-stone-300">{'function Counter() {'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'  const value = useFx(count)'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'  return <div>{value}</div>'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'}'}</span>
                </code>
              </pre>
            </div>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  <span className="text-stone-600">{'// useComputed — derived values, auto-tracked'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'const doubled = useComputed(() => count() * 2)'}</span>
                </code>
              </pre>
            </div>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  <span className="text-stone-600">{'// useRun — side effects, auto-tracked'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'useRun(() => {'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'  console.log(\'Count changed:\', count())'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'}'}</span>
                  <span className="text-stone-300">{')'}</span>
                </code>
              </pre>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">AI hooks</h2>
            <p className="text-sm text-stone-500 leading-relaxed">
              Streaming chat and structured object generation with built-in state management.
            </p>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  <span className="text-stone-600">{'// useAiChat — streaming chat'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'import { useAiChat } from \'@flickjs/react/ai\''}</span>
                  {'\n\n'}
                  <span className="text-stone-300">{'const { messages, input, handleInputChange, handleSubmit } ='}</span>
                  {'\n'}
                  <span className="text-stone-300">{'  useAiChat({ api: \'/api/chat\' })'}</span>
                </code>
              </pre>
            </div>
            <div className="border border-stone-800 rounded p-4 leading-relaxed text-sm">
              <pre>
                <code>
                  <span className="text-stone-600">{'// useAiObject — structured output with Zod'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'import { useAiObject } from \'@flickjs/react/ai\''}</span>
                  {'\n'}
                  <span className="text-stone-300">{'import { z } from \'zod\''}</span>
                  {'\n\n'}
                  <span className="text-stone-300">{'const { object, isLoading, submit } = useAiObject({'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'  api: \'/api/recipe\','}</span>
                  {'\n'}
                  <span className="text-stone-300">{'  schema: z.object({ name: z.string(), ingredients: z.array(z.string()) }),'}</span>
                  {'\n'}
                  <span className="text-stone-300">{'}'}</span>
                  <span className="text-stone-300">{')'}</span>
                </code>
              </pre>
            </div>
          </div>
        </section>

        <section className="container pb-12 lg:pb-16">
          <div className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tighter text-foreground">Features</h2>
            <div className="flex flex-col gap-4">
              {[
                {
                  title: 'Surgical re-renders',
                  description:
                    'Only components reading a changed signal re-render. No cascading updates through the component tree.',
                },
                {
                  title: 'Auto-tracked dependencies',
                  description:
                    'No dependency arrays. useComputed and useRun track signals automatically.',
                },
                {
                  title: 'AI streaming',
                  description:
                    'Built-in hooks for chat and structured output with streaming support.',
                },
                {
                  title: 'Server-side agents',
                  description:
                    'Define agents, tools, and routers. Works with Express, Node.js, and Vercel Functions.',
                },
                {
                  title: 'Type safe',
                  description:
                    'Full TypeScript support with Zod schema validation for structured AI outputs.',
                },
                {
                  title: 'Tree-shakeable',
                  description:
                    'Side-effects free. Import only what you use.',
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
      </main>
      <Footer />
    </div>
  );
};

export default ReactPage;
