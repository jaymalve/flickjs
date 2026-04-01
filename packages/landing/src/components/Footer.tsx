const Footer = () => {
  return (
    <footer className="container pb-20 lg:pb-32">
      <div className="flex items-center gap-4 text-sm text-stone-600">
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
        <a
          href="https://x.com/jaydotdev"
          target="_blank"
          rel="noopener noreferrer"
          className="link"
        >
          twitter
        </a>
      </div>
    </footer>
  );
};

export default Footer;
