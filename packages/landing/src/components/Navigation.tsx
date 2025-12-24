import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Check, Copy, Menu, X } from "lucide-react";

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
        <a
          href="/"
          className="text-lg font-semibold tracking-tight text-foreground"
        >
          Flick
        </a>

        {/* Desktop Navigation */}
        <div className="hidden md:flex items-center gap-6">
          <a href="#docs" className="nav-link text-sm">
            Docs
          </a>
          <a
            href="https://github.com"
            target="_blank"
            rel="noopener noreferrer"
            className="nav-link text-sm"
          >
            GitHub
          </a>
          <a
            href="https://npmjs.com"
            target="_blank"
            rel="noopener noreferrer"
            className="nav-link text-sm"
          >
            NPM
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
            <a href="#docs" className="nav-link text-sm py-2">
              Docs
            </a>
            <a
              href="https://github.com"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-link text-sm py-2"
            >
              GitHub
            </a>
            <a
              href="https://npmjs.com"
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
