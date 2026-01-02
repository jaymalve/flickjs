declare type Run = () => void;

export declare type Fx<T> = (() => T) & {
  set: (next: T | ((v: T) => T)) => void;
};

export declare function fx<T>(value: T): Fx<T>;

export declare function run(fn: Run): void;

export declare function mount(App: () => Node, el: HTMLElement): void;

export declare function Suspense(props: SuspenseProps): Node;

export declare function lazy<P extends Record<string, any> = {}>(
  loader: LazyLoader<P>
): LazyComponent<P>;
export interface SuspenseProps {
  fallback: Node | (() => Node);
  children?: Node | (() => Node);
}

export declare function query<T>(fetcher: () => Promise<T>): Query<T>;
export declare function query<S, T>(
  source: () => S,
  fetcher: (source: S) => Promise<T>
): Query<T>;

export declare function renderList<T>(
  parent: Node,
  anchor: Node,
  getItems: () => T[],
  mapFn: (item: T, index: number) => Node,
  getKey: (item: T, index: number) => string | number = (_, i) => i
): void;

export declare function getCurrentSuspense(): Suspense | null;

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
