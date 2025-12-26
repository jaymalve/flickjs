import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Check, Copy, Menu, X } from "lucide-react";

const GitHubIcon = ({ className }: { className?: string }) => (
  <svg viewBox="0 0 24 24" fill="currentColor" className={className}>
    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
  </svg>
);

const Navigation = () => {
  const [copied, setCopied] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const installCommand = "bunx create-flick-app my-app";

  const handleCopy = async () => {
    await navigator.clipboard.writeText(installCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 border-b border-input backdrop-blur-md bg-background/80">
      <div className="container flex h-14 items-center justify-between">
        {/* Logo */}
        <a href="/" className="flex items-center">
          <svg
            width="24"
            height="24"
            viewBox="0 0 32 32"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            className="text-foreground"
          >
            <defs>
              <linearGradient id="cubeGrad" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="currentColor" stopOpacity="1" />
                <stop
                  offset="100%"
                  stopColor="currentColor"
                  stopOpacity="0.4"
                />
              </linearGradient>
              <linearGradient id="topFaceNav" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="currentColor" stopOpacity="0.15" />
                <stop
                  offset="100%"
                  stopColor="currentColor"
                  stopOpacity="0.08"
                />
              </linearGradient>
              <linearGradient
                id="rightFaceNav"
                x1="0%"
                y1="0%"
                x2="100%"
                y2="100%"
              >
                <stop offset="0%" stopColor="currentColor" stopOpacity="0.08" />
                <stop
                  offset="100%"
                  stopColor="currentColor"
                  stopOpacity="0.04"
                />
              </linearGradient>
              <linearGradient
                id="leftFaceNav"
                x1="100%"
                y1="0%"
                x2="0%"
                y2="100%"
              >
                <stop offset="0%" stopColor="currentColor" stopOpacity="0.05" />
                <stop
                  offset="100%"
                  stopColor="currentColor"
                  stopOpacity="0.02"
                />
              </linearGradient>
            </defs>
            {/* Cube faces with gradient fills */}
            <path d="M16 4L26 10L16 16L6 10Z" fill="url(#topFaceNav)" />
            <path d="M16 16L26 10L26 22L16 28Z" fill="url(#rightFaceNav)" />
            <path d="M6 10L16 16L16 28L6 22Z" fill="url(#leftFaceNav)" />
            {/* Cube outline */}
            <path
              d="M16 4L6 10V22L16 28L26 22V10L16 4Z"
              stroke="url(#cubeGrad)"
              strokeWidth="1.5"
              strokeLinejoin="round"
              fill="none"
            />
            <path
              d="M6 10L16 16L26 10"
              stroke="url(#cubeGrad)"
              strokeWidth="1.5"
              strokeLinejoin="round"
            />
            <line
              x1="16"
              y1="16"
              x2="16"
              y2="28"
              stroke="url(#cubeGrad)"
              strokeWidth="1.5"
            />
            {/* Enhanced chevron - larger and bolder */}
            <path
              d="M20 6L26 10L20 14"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinejoin="round"
              fill="none"
            />
          </svg>
        </a>

        {/* Desktop Navigation */}
        <div className="hidden md:flex items-center gap-6">
          <a href="/docs" className="nav-link text-sm">
            Docs
          </a>
          <a
            href="https://github.com/jaymalve/flickjs"
            target="_blank"
            rel="noopener noreferrer"
            className="nav-link"
            aria-label="GitHub"
          >
            <GitHubIcon className="h-4 w-4" />
          </a>
          <a
            href="https://www.npmjs.com/package/@flickjs/runtime"
            target="_blank"
            rel="noopener noreferrer"
            className="nav-link text-sm"
          >
            npm
          </a>

          {/* Copy Command Button */}
          <Button
            variant="copy"
            size="sm"
            onClick={handleCopy}
            className="font-mono text-xs gap-3"
          >
            <span className="text-muted-foreground">$</span>
            <span>{installCommand}</span>
            {copied ? (
              <Check className="h-3 w-3 text-accent" />
            ) : (
              <Copy className="h-3 w-3" />
            )}
          </Button>
        </div>

        {/* Mobile Menu Button */}
        <button
          className="md:hidden p-2 text-muted-foreground hover:text-foreground transition-colors"
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
        >
          {mobileMenuOpen ? (
            <X className="h-5 w-5" />
          ) : (
            <Menu className="h-5 w-5" />
          )}
        </button>
      </div>

      {/* Mobile Menu */}
      {mobileMenuOpen && (
        <div className="md:hidden border-t border-input bg-background/95 backdrop-blur-md">
          <div className="container py-4 flex flex-col gap-4">
            <a href="/docs" className="nav-link text-sm py-2">
              Docs
            </a>
            <a
              href="https://github.com/jaymalve/flickjs"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-link py-2"
              aria-label="GitHub"
            >
              <GitHubIcon className="h-4 w-4" />
            </a>
            <a
              href="https://www.npmjs.com/package/@flickjs/runtime"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-link text-sm py-2"
            >
              NPM
            </a>
            <Button
              variant="copy"
              size="sm"
              onClick={handleCopy}
              className="font-mono text-xs gap-3 w-full justify-center"
            >
              <span className="text-muted-foreground">$</span>
              <span>{installCommand}</span>
              {copied ? (
                <Check className="h-3 w-3 text-accent" />
              ) : (
                <Copy className="h-3 w-3" />
              )}
            </Button>
          </div>
        </div>
      )}
    </nav>
  );
};

export default Navigation;
