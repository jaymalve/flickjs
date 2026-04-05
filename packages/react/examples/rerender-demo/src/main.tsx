import { scan } from 'react-scan/all-environments';
import { createRoot } from 'react-dom/client';
import { App } from './App';

scan();

// Global reset matching FlickJS landing page
const style = document.createElement('style');
style.textContent = `
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  html { scroll-behavior: smooth; }
  body {
    background: #000;
    color: #fafafa;
    font-family:
      'Geist Mono',
      ui-monospace,
      SFMono-Regular,
      'SF Mono',
      Menlo,
      Monaco,
      Consolas,
      monospace;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    min-height: 100vh;
  }
  ::selection { background: rgba(255,255,255,0.12); }
`;
document.head.appendChild(style);

createRoot(document.getElementById('root')!).render(<App />);
