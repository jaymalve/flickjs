const Footer = () => {
  return (
    <footer className="py-8 border-t border-input">
      <div className="container">
        <div className="flex flex-col md:flex-row items-center justify-center gap-4">
          {/* Links */}
          <div className="flex items-center gap-6">
            <a href="/docs" className="nav-link text-xs">
              Docs
            </a>
            <a
              href="https://github.com/jaymalve/flickjs"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-link text-xs"
            >
              GitHub
            </a>
            <a
              href="https://www.npmjs.com/package/@flickjs/runtime"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-link text-xs"
            >
              NPM
            </a>
            <a
              href="https://x.com/jaydotdev"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-link text-xs"
            >
              Twitter
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
