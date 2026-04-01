import { createRoot } from "react-dom/client";
import { App } from "./App";

// Global reset matching FlickJS landing page
const style = document.createElement("style");
style.textContent = `
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  html { scroll-behavior: smooth; }
  body {
    background: #000;
    color: #fafafa;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    min-height: 100vh;
  }
  ::selection { background: rgba(255,255,255,0.12); }
`;
document.head.appendChild(style);

createRoot(document.getElementById("root")!).render(<App />);
