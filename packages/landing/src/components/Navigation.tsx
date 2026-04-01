const Navigation = () => {
  return (
    <nav className="container pt-7 pb-4 lg:pt-10">
      <div className="flex items-center justify-between">
        <a href="/" className="flex items-center gap-2">
          <svg
            width="24"
            height="24"
            viewBox="0 0 32 32"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            className="text-foreground"
          >
            <path
              d="M16 4L6 10V22L16 28L26 22V10L16 4Z"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinejoin="round"
              fill="none"
            />
            <path
              d="M6 10L16 16L26 10"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinejoin="round"
            />
            <line x1="16" y1="16" x2="16" y2="28" stroke="currentColor" strokeWidth="1.5" />
            <path
              d="M20 6L26 10L20 14"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinejoin="round"
              fill="none"
            />
          </svg>
          <span className="text-sm font-semibold tracking-tighter text-foreground">flickjs</span>
        </a>

        <div className="flex items-center gap-4 text-sm text-stone-400">
          <a href="/docs" className="link">
            docs
          </a>
          <a
            href="https://github.com/user/flickjs"
            target="_blank"
            rel="noopener noreferrer"
            className="link"
          >
            github
          </a>
          <a
            href="https://www.npmjs.com/package/@flickjs/runtime"
            target="_blank"
            rel="noopener noreferrer"
            className="link"
          >
            npm
          </a>
        </div>
      </div>
    </nav>
  );
};

export default Navigation;
