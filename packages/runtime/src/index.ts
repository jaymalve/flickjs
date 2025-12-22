type Effect = () => void;

let activeEffect: Effect | null = null;

export function signal<T>(value: T) {
  const subs = new Set<Effect>();

  function read() {
    if (activeEffect) subs.add(activeEffect);
    return value;
  }

  read.set = (next: T | ((v: T) => T)) => {
    value = typeof next === "function" ? (next as any)(value) : next;
    subs.forEach((fn) => fn());
  };

  return read as (() => T) & { set: typeof read.set };
}

export function effect(fn: Effect) {
  const run = () => {
    activeEffect = run;
    fn();
    activeEffect = null;
  };
  run();
}

export function renderList<T>(
  parent: Node,
  anchor: Node,
  getItems: () => T[],
  mapFn: (item: T, index: number) => Node,
  getKey: (item: T, index: number) => string | number = (_, i) => i
) {
  const nodeMap = new Map<string | number, Node>();
  let currentKeys: (string | number)[] = [];

  effect(() => {
    const items = getItems();
    const newKeys = items.map((item, i) => getKey(item, i));
    const newNodeMap = new Map<string | number, Node>();

    // Build new nodes, reusing existing where possible
    const newNodes = items.map((item, i) => {
      const key = newKeys[i];
      let node = nodeMap.get(key);

      if (!node) {
        // Create new node for this key
        node = mapFn(item, i);
      }

      newNodeMap.set(key, node);
      return node;
    });

    // Remove nodes that are no longer in the list
    currentKeys.forEach((key) => {
      if (!newNodeMap.has(key)) {
        const node = nodeMap.get(key);
        if (node && node.parentNode) {
          node.parentNode.removeChild(node);
        }
      }
    });

    // Insert nodes in correct order
    let nextSibling: Node | null = anchor;
    for (let i = newNodes.length - 1; i >= 0; i--) {
      const node = newNodes[i];
      if (node.nextSibling !== nextSibling) {
        parent.insertBefore(node, nextSibling);
      }
      nextSibling = node;
    }

    // Update state
    nodeMap.clear();
    newNodeMap.forEach((v, k) => nodeMap.set(k, v));
    currentKeys = newKeys;
  });
}

export function mount(App: () => Node, el: HTMLElement) {
  el.appendChild(App());
}

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

// This ensures JSX types are loaded when the module is imported
// Users don't need to do anything - types are automatically available
export const jsxTypes = Symbol("jsx-types");

export { Suspense, getCurrentSuspense, resource, lazy } from "./suspense";
export type { SuspenseContext, SuspenseProps, Resource } from "./suspense";
