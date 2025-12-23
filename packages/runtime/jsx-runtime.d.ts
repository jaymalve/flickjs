declare type Effect = () => void;

export declare function signal<T>(value: T): (() => T) & {
  set: (next: T | ((v: T) => T)) => void;
};

export declare function effect(fn: Effect): void;

export declare function mount(App: () => Node, el: HTMLElement): void;

export declare function Suspense(props: SuspenseProps): Node;
export interface SuspenseProps {
  fallback: Node | (() => Node);
  children?: Node | (() => Node);
}

export declare function resource<T>(fetcher: () => Promise<T>): Resource<T>;
export declare function resource<S, T>(
  source: () => S,
  fetcher: (source: S) => Promise<T>
): Resource<T>;

// JSX Type Definitions for Flick Framework - automatically available when imported
declare global {
  namespace JSX {
    interface Element extends Node {}

    interface IntrinsicElements {
      // HTML elements with flexible typing for custom JSX compiler
      div: any;
      h1: any;
      h2: any;
      h3: any;
      h4: any;
      h5: any;
      h6: any;
      p: any;
      span: any;
      button: any;
      input: any;
      textarea: any;
      select: any;
      option: any;
      ul: any;
      ol: any;
      li: any;
      a: any;
      img: any;
      pre: any;
      code: any;
      br: any;
      hr: any;

      // Allow custom element names
      [elemName: string]: any;
    }
  }
}

export declare const jsxTypes: unique symbol;

export {};
