import { Button } from "@/components/ui/button";
import { ArrowRight } from "lucide-react";

const Hero = () => {
  return (
    <section className="relative pt-32 pb-20 md:pt-40 md:pb-32">
      {/* Subtle gradient background */}
      <div className="absolute inset-0 gradient-bg" />

      <div className="container relative">
        <div className="grid lg:grid-cols-2 gap-12 lg:gap-16 items-center">
          {/* Left side - Text content */}
          <div className="text-left">
            {/* Badge */}
            {/* <div className="animate-fade-up inline-flex items-center gap-2 px-3 py-1 mb-8 text-xs font-medium text-muted-foreground border border-input rounded-full">
              <span className="w-1.5 h-1.5 rounded-full bg-accent" />
              Now available on NPM
            </div> */}

            {/* Headline */}
            <h1 className="animate-fade-up-delay-1 text-2xl md:text-5xl lg:text-3xl font-bold tracking-tighter text-foreground leading-[1.1] mb-6">
              Deterministic Reactivity at Scale.
              <br /> {/* <span className="text-accent">UI framework</span>. */}
            </h1>

            {/* Sub-headline */}
            <p className="animate-fade-up-delay-2 text-lg md:text-xl text-muted-foreground max-w-md mb-10 leading-relaxed">
              Ship what matters.
            </p>

            {/* CTAs */}
            <div className="animate-fade-up-delay-3 flex flex-col sm:flex-row items-start gap-4">
              <Button variant="hero" size="default" className="group" asChild>
                <a href="/docs">
                  Get Started
                  <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
                </a>
              </Button>
              <Button variant="heroOutline" size="default" asChild>
                <a
                  href="https://github.com/jaymalve/flickjs"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  View on GitHub
                </a>
              </Button>
            </div>

            {/* Stats */}
            {/* <div className="animate-fade-up-delay-3 flex items-center gap-8 mt-12 pt-8 border-t border-input">
              <div className="text-left">
                <div className="text-2xl md:text-3xl font-bold text-foreground tracking-tight">
                  0
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Dependencies
                </div>
              </div>
              <div className="w-px h-10 bg-input" />
              <div className="text-left">
                <div className="text-2xl md:text-3xl font-bold text-foreground tracking-tight">
                  0
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Virtual DOM
                </div>
              </div>
              <div className="w-px h-10 bg-input" />
              <div className="text-left">
                <div className="text-2xl md:text-3xl font-bold text-foreground tracking-tight">
                  ∞
                </div>
                <div className="text-xs text-muted-foreground mt-1">
                  Possibilities
                </div>
              </div>
            </div> */}
          </div>

          {/* Right side - Code block */}
          <div className="animate-fade-up-delay-2 lg:animate-fade-up-delay-1">
            <div className="code-block rounded overflow-hidden">
              {/* Window header */}
              <div className="flex items-center gap-2 px-4 py-3 border-b border-input">
                <div className="flex gap-1.5">
                  <div className="w-3 h-3 rounded-full bg-muted-foreground/20" />
                  <div className="w-3 h-3 rounded-full bg-muted-foreground/20" />
                  <div className="w-3 h-3 rounded-full bg-muted-foreground/20" />
                </div>
                <span className="ml-2 text-xs text-muted-foreground font-mono">
                  Counter.jsx
                </span>
                {/* <span className="ml-auto text-xs font-mono">
                  <span className="text-green-400/70">+6</span>
                  <span className="text-muted-foreground mx-1">/</span>
                  <span className="text-red-400/70">-12</span>
                </span> */}
              </div>

              {/* Code content */}
              <div className="py-3 md:py-4 overflow-x-auto">
                <pre className="text-xs font-mono leading-relaxed">
                  <code>
                    {/* React code - removed */}
                    <RemovedLine>
                      <Keyword>import</Keyword>
                      <Plain> {"{"} </Plain>
                      <Variable>useState</Variable>
                      <Plain>, </Plain>
                      <Variable>useEffect</Variable>
                      <Plain>, </Plain>
                      <Variable>useMemo</Variable>
                      <Plain> {"}"} </Plain>
                      <Keyword>from</Keyword>
                      <String> 'react'</String>
                      <Plain>;</Plain>
                    </RemovedLine>
                    <RemovedLine />
                    <RemovedLine>
                      <Keyword>const</Keyword>
                      <Plain> [count, setCount] = </Plain>
                      <Function>useState</Function>
                      <Plain>(</Plain>
                      <Number>0</Number>
                      <Plain>);</Plain>
                    </RemovedLine>
                    <RemovedLine>
                      <Keyword>const</Keyword>
                      <Plain> doubled = </Plain>
                      <Function>useMemo</Function>
                      <Plain>{"(() => count * 2, [count]);"}</Plain>
                    </RemovedLine>
                    <RemovedLine />
                    <RemovedLine>
                      <Function>useEffect</Function>
                      <Plain>{"(() => {"}</Plain>
                    </RemovedLine>
                    <RemovedLine>
                      <Plain>console.</Plain>
                      <Function>log</Function>
                      <Plain>(</Plain>
                      <String>'Count:'</String>
                      <Plain>, count);</Plain>
                    </RemovedLine>
                    <RemovedLine>
                      <Plain>{"}, [count]);"}</Plain>
                    </RemovedLine>
                    <RemovedLine />
                    <RemovedLine>
                      <Keyword>const</Keyword>
                      <Plain> increment = </Plain>
                      <Function>useCallback</Function>
                      <Plain>{"(() => {"}</Plain>
                    </RemovedLine>
                    <RemovedLine>
                      <Function>setCount</Function>
                      <Plain>{"(prev => prev + 1);"}</Plain>
                    </RemovedLine>
                    <RemovedLine>
                      <Plain>{"}, []);"}</Plain>
                    </RemovedLine>

                    {/* FlickJS code - added */}
                    <AddedLine>
                      <Keyword>import</Keyword>
                      <Plain> {"{"} </Plain>
                      <Variable>fx</Variable>
                      <Plain>, </Plain>
                      <Variable>run</Variable>
                      <Plain> {"}"} </Plain>
                      <Keyword>from</Keyword>
                      <String> '@flickjs/runtime'</String>
                      <Plain>;</Plain>
                    </AddedLine>
                    <AddedLine />
                    <AddedLine>
                      <Keyword>const</Keyword>
                      <Plain> count = </Plain>
                      <Function>fx</Function>
                      <Plain>(</Plain>
                      <Number>0</Number>
                      <Plain>);</Plain>
                    </AddedLine>
                    <AddedLine>
                      <Keyword>const</Keyword>
                      <Plain> doubled = {"() => count() * "}</Plain>
                      <Number>2</Number>
                      <Plain>;</Plain>
                    </AddedLine>
                    <AddedLine />
                    <AddedLine>
                      <Function>run</Function>
                      <Plain>{"(() => console."}</Plain>
                      <Function>log</Function>
                      <Plain>(</Plain>
                      <String>'Count:'</String>
                      <Plain>, count()));</Plain>
                    </AddedLine>
                  </code>
                </pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

// Syntax highlighting components
const Keyword = ({ children }: { children: React.ReactNode }) => (
  <span className="text-[#E4E4E7]">{children}</span>
);

const Function = ({ children }: { children: React.ReactNode }) => (
  <span className="text-[#FAFAFA]">{children}</span>
);

const Variable = ({ children }: { children: React.ReactNode }) => (
  <span className="text-[#A1A1AA]">{children}</span>
);

const String = ({ children }: { children: React.ReactNode }) => (
  <span className="text-[#D1D5DB]">{children}</span>
);

const Number = ({ children }: { children: React.ReactNode }) => (
  <span className="text-[#A1A1AA]">{children}</span>
);

const Plain = ({ children }: { children: React.ReactNode }) => (
  <span className="text-[#71717A]">{children}</span>
);

// Diff line components
const RemovedLine = ({
  children,
  indent = 0,
}: {
  children?: React.ReactNode;
  indent?: number;
}) => (
  <div
    className="min-h-[1.25rem] bg-red-500/10 -mx-3 md:-mx-4 px-3 md:px-4"
    style={{ paddingLeft: `calc(${indent * 0.1}rem + 1rem)` }}
  >
    <span className="text-red-400/70 mr-2">-</span>
    {children}
  </div>
);

const AddedLine = ({
  children,
  indent = 0,
}: {
  children?: React.ReactNode;
  indent?: number;
}) => (
  <div
    className="min-h-[1.25rem] bg-green-500/10 -mx-3 md:-mx-4 px-3 md:px-4"
    style={{ paddingLeft: `calc(${indent * 0.5}rem + 1rem)` }}
  >
    <span className="text-green-400/70 mr-2">+</span>
    {children}
  </div>
);

export default Hero;
