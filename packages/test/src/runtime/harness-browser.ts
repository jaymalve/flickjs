/**
 * Browser-only harness: React rendering, DOM events, waitFor, act.
 */
import { type ReactElement, StrictMode, act as reactAct } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import {
  installCoreGlobals,
  clearHarness,
  runAllTests,
  getTestReport,
  setHarnessContext,
  afterEach
} from './harness-core.js';

const roots: Root[] = [];

async function defaultAct(fn: () => void | Promise<void>): Promise<void> {
  await reactAct(async () => {
    await fn();
  });
}

let actImpl = defaultAct;

export function cleanup(): void {
  for (const r of roots.splice(0)) {
    try {
      r.unmount();
    } catch {
      /* ignore */
    }
  }
  document.body.replaceChildren();
}

export function render(element: ReactElement): { container: HTMLElement; unmount: () => void } {
  const container = document.createElement('div');
  document.body.appendChild(container);
  const root = createRoot(container);
  roots.push(root);
  root.render(<StrictMode>{element}</StrictMode>);
  return {
    container,
    unmount: () => {
      root.unmount();
      container.remove();
      const i = roots.indexOf(root);
      if (i >= 0) roots.splice(i, 1);
    }
  };
}

export function installBrowserHarness(testFile: string): void {
  clearHarness();
  setHarnessContext(testFile, 'browser');
  installCoreGlobals(globalThis);
  afterEach(cleanup);

  const g = globalThis as Record<string, unknown>;
  g.render = render;
  g.cleanup = cleanup;
  g.screen = screen;
  g.fireEvent = fireEvent;
  g.waitFor = waitFor;
  g.act = act;
}

function cssEscape(s: string): string {
  if (typeof CSS !== 'undefined' && 'escape' in CSS) {
    return CSS.escape(s);
  }
  return s.replace(/["\\]/g, '\\$&');
}

function matchTextContent(content: string, text: string | RegExp): boolean {
  const norm = content.replace(/\s+/g, ' ').trim();
  if (typeof text === 'string') return norm.includes(text);
  return text.test(norm);
}

function matchesText(el: Element, text: string | RegExp): boolean {
  const t = el.textContent ?? '';
  return matchTextContent(t, text);
}

const screen = {
  getByText(text: string | RegExp): HTMLElement {
    const all = document.body.querySelectorAll('*');
    for (const el of all) {
      if (matchesText(el, text) && el.childElementCount === 0 && el.textContent?.trim()) {
        return el as HTMLElement;
      }
    }
    for (const el of all) {
      if (matchesText(el, text)) return el as HTMLElement;
    }
    throw new Error(`getByText: no match for ${String(text)}`);
  },
  getByTestId(id: string): HTMLElement {
    const el = document.querySelector(`[data-testid="${cssEscape(id)}"]`);
    if (!el) throw new Error(`getByTestId: ${id}`);
    return el as HTMLElement;
  },
  queryByText(text: string | RegExp): HTMLElement | null {
    try {
      return screen.getByText(text);
    } catch {
      return null;
    }
  },
  queryByTestId(id: string): HTMLElement | null {
    return document.querySelector(`[data-testid="${cssEscape(id)}"]`) as HTMLElement | null;
  }
};

async function act(fn: () => void | Promise<void>): Promise<void> {
  await actImpl(fn);
}

export const fireEvent = {
  click(element: Element): void {
    element.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, view: window }));
  },
  dblClick(element: Element): void {
    element.dispatchEvent(
      new MouseEvent('dblclick', { bubbles: true, cancelable: true, view: window })
    );
  },
  change(element: HTMLInputElement, options: { target: { value: string } }): void {
    element.value = options.target.value;
    element.dispatchEvent(new Event('input', { bubbles: true }));
    element.dispatchEvent(new Event('change', { bubbles: true }));
  },
  input(element: HTMLInputElement, options: { target: { value: string } }): void {
    element.value = options.target.value;
    element.dispatchEvent(new Event('input', { bubbles: true }));
  },
  keyDown(element: Element, options: { key: string; code?: string }): void {
    element.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: options.key,
        code: options.code ?? options.key,
        bubbles: true,
        cancelable: true,
        view: window
      })
    );
  },
  keyUp(element: Element, options: { key: string; code?: string }): void {
    element.dispatchEvent(
      new KeyboardEvent('keyup', {
        key: options.key,
        code: options.code ?? options.key,
        bubbles: true,
        cancelable: true,
        view: window
      })
    );
  },
  keyPress(element: Element, options: { key: string; code?: string }): void {
    element.dispatchEvent(
      new KeyboardEvent('keypress', {
        key: options.key,
        code: options.code ?? options.key,
        bubbles: true,
        cancelable: true,
        view: window
      })
    );
  },
  focus(element: Element): void {
    (element as HTMLElement).focus();
    element.dispatchEvent(new FocusEvent('focus', { bubbles: true }));
  },
  blur(element: Element): void {
    (element as HTMLElement).blur();
    element.dispatchEvent(new FocusEvent('blur', { bubbles: true }));
  },
  submit(element: Element): void {
    element.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
  },
  mouseEnter(element: Element): void {
    element.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true, view: window }));
  },
  mouseLeave(element: Element): void {
    element.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true, view: window }));
  },
  scroll(element: Element): void {
    element.dispatchEvent(new Event('scroll', { bubbles: true }));
  }
};

export async function waitFor(
  assertion: () => void,
  options?: { timeout?: number; interval?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 1000;
  const interval = options?.interval ?? 50;
  const start = Date.now();
  for (;;) {
    try {
      assertion();
      return;
    } catch {
      if (Date.now() - start > timeout) throw new Error(`waitFor: timeout after ${timeout}ms`);
      await new Promise((r) => setTimeout(r, interval));
    }
  }
}

export { runAllTests, getTestReport };
