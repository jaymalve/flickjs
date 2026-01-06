type Run = () => void;

export type Fx<T> = (() => T) & { set: (next: T | ((v: T) => T)) => void };

let activeRun: Run | null = null;

/*
 * Instrumentation Hooks (used by DevTools)
 */

interface RuntimeHooks {
  onEffectStart?: (runId: number) => void;
  onEffectEnd?: (
    runId: number,
    duration: number,
    domNodes: Set<Node>,
    componentName?: string
  ) => void;
  onSignalUpdate?: (
    fxId: number,
    prevValue: unknown,
    nextValue: unknown,
    name?: string
  ) => void;
}

let hooks: RuntimeHooks = {};
let fxIdCounter = 0;
let runIdCounter = 0;

// Maps to track metadata
const fxIdMap = new WeakMap<Function, number>();
const fxNameMap = new WeakMap<Function, string>();
const runIdMap = new WeakMap<Function, number>();

/**
 * Register instrumentation hooks (called by DevTools)
 */
export function __registerHooks(newHooks: RuntimeHooks): void {
  hooks = newHooks;
}

/**
 * Unregister instrumentation hooks
 */
export function __unregisterHooks(): void {
  hooks = {};
}

/**
 * Get signal name (for devtools)
 */
export function __getFxName(fx: Function): string | undefined {
  return fxNameMap.get(fx);
}

export function fx<T>(value: T, name?: string): Fx<T> {
  const subs = new Set<Run>();
  const fxId = fxIdCounter++;

  function read() {
    if (activeRun) subs.add(activeRun);
    return value;
  }

  read.set = (next: T | ((v: T) => T)) => {
    const prevValue = value;
    value = typeof next === "function" ? (next as any)(value) : next;

    // Notify devtools hook
    if (hooks.onSignalUpdate) {
      hooks.onSignalUpdate(fxId, prevValue, value, name);
    }

    subs.forEach((fn) => fn());
  };

  // Store fxId and name for devtools lookup
  fxIdMap.set(read, fxId);
  if (name) {
    fxNameMap.set(read, name);
  }

  return read as (() => T) & { set: typeof read.set };
}

export function run(fn: Run, componentName?: string) {
  const runId = runIdCounter++;

  const execute = () => {
    const startTime = performance.now();
    const domNodes = new Set<Node>();

    // Set up DOM mutation observer if hooks are registered
    let observer: MutationObserver | null = null;
    if (hooks.onEffectEnd && typeof MutationObserver !== "undefined") {
      observer = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          domNodes.add(mutation.target);
          mutation.addedNodes.forEach((node) => domNodes.add(node));
        }
      });
      try {
        if (document.body) {
          observer.observe(document.body, {
            childList: true,
            subtree: true,
            attributes: true,
            characterData: true,
          });
        }
      } catch {
        // Ignore if DOM not ready
      }
    }

    // Notify start
    if (hooks.onEffectStart) {
      hooks.onEffectStart(runId);
    }

    activeRun = execute;
    fn();
    activeRun = null;

    // Stop observing and collect any pending mutations
    if (observer) {
      // takeRecords() synchronously returns pending mutations before they're delivered async
      const pendingMutations = observer.takeRecords();
      for (const mutation of pendingMutations) {
        domNodes.add(mutation.target);
        mutation.addedNodes.forEach((node) => domNodes.add(node));
      }
      observer.disconnect();
    }

    // Notify end with timing, DOM nodes, and component name
    if (hooks.onEffectEnd) {
      const duration = performance.now() - startTime;
      hooks.onEffectEnd(runId, duration, domNodes, componentName);
    }
  };

  // Store runId for devtools lookup
  runIdMap.set(execute, runId);

  execute();
}

/**
 * Compute Longest Increasing Subsequence using binary search.
 * Returns indices into the input array representing the LIS.
 * Time: O(n log n), Space: O(n)
 */
function longestIncreasingSubsequence(arr: number[]): number[] {
  const n = arr.length;
  if (n === 0) return [];

  const tails: number[] = [];
  const predecessors: number[] = new Array(n);

  for (let i = 0; i < n; i++) {
    const val = arr[i];
    let lo = 0,
      hi = tails.length;

    while (lo < hi) {
      const mid = (lo + hi) >>> 1;
      if (arr[tails[mid]] < val) lo = mid + 1;
      else hi = mid;
    }

    if (lo === tails.length) tails.push(i);
    else tails[lo] = i;

    predecessors[i] = lo > 0 ? tails[lo - 1] : -1;
  }

  const result: number[] = new Array(tails.length);
  let k = tails[tails.length - 1];
  for (let i = tails.length - 1; i >= 0; i--) {
    result[i] = k;
    k = predecessors[k];
  }

  return result;
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

  run(() => {
    const items = getItems();
    const newLen = items.length;
    const oldLen = currentKeys.length;

    // Fast path: empty new list - remove all nodes
    if (newLen === 0) {
      for (const key of currentKeys) {
        const node = nodeMap.get(key);
        if (node?.parentNode) node.parentNode.removeChild(node);
      }
      nodeMap.clear();
      currentKeys = [];
      return;
    }

    const newKeys = items.map((item, i) => getKey(item, i));

    // Fast path: empty old list (first render) - batch insert
    if (oldLen === 0) {
      const fragment = document.createDocumentFragment();
      for (let i = 0; i < newLen; i++) {
        const key = newKeys[i];
        const node = mapFn(items[i], i);
        nodeMap.set(key, node);
        fragment.appendChild(node);
      }
      parent.insertBefore(fragment, anchor);
      currentKeys = newKeys;
      return;
    }

    // Build old key -> old index map for O(1) position lookup
    const oldKeyToIdx = new Map<string | number, number>();
    for (let i = 0; i < oldLen; i++) {
      oldKeyToIdx.set(currentKeys[i], i);
    }

    // Build new nodes, track sources (old indices), detect if moves needed
    const newNodeMap = new Map<string | number, Node>();
    const newNodes: Node[] = new Array(newLen);
    const sources: number[] = new Array(newLen).fill(-1); // -1 = new node
    let moved = false;
    let maxOldIdx = -1;

    for (let i = 0; i < newLen; i++) {
      const key = newKeys[i];
      let node = nodeMap.get(key);

      if (node) {
        // Reuse existing node - track its old position
        const oldIdx = oldKeyToIdx.get(key)!;
        sources[i] = oldIdx;
        // If old index is less than max seen, elements are out of order
        if (oldIdx < maxOldIdx) moved = true;
        else maxOldIdx = oldIdx;
      } else {
        // Create new node
        node = mapFn(items[i], i);
      }

      newNodes[i] = node;
      newNodeMap.set(key, node);
    }

    // Remove nodes no longer in list
    for (const key of currentKeys) {
      if (!newNodeMap.has(key)) {
        const node = nodeMap.get(key);
        if (node?.parentNode) node.parentNode.removeChild(node);
      }
    }

    // Position nodes in DOM
    if (!moved) {
      // Fast path: no reordering needed - only insert new nodes
      let nextSibling: Node | null = anchor;
      for (let i = newLen - 1; i >= 0; i--) {
        const node = newNodes[i];
        if (sources[i] === -1) {
          // New node - insert it
          parent.insertBefore(node, nextSibling);
        }
        nextSibling = node;
      }
    } else {
      // Use LIS to minimize DOM moves
      // Build array of old indices for existing nodes only
      const toMove: number[] = [];
      const toMoveNewIdx: number[] = [];

      for (let i = 0; i < newLen; i++) {
        if (sources[i] !== -1) {
          toMove.push(sources[i]);
          toMoveNewIdx.push(i);
        }
      }

      // Find LIS - these nodes don't need to move
      const lisIndices = longestIncreasingSubsequence(toMove);
      const inLIS = new Set(lisIndices.map((idx) => toMoveNewIdx[idx]));

      // Insert from end to start, only moving nodes not in LIS
      let nextSibling: Node | null = anchor;
      for (let i = newLen - 1; i >= 0; i--) {
        const node = newNodes[i];
        if (sources[i] === -1 || !inLIS.has(i)) {
          // New node or node not in LIS - needs insertion
          parent.insertBefore(node, nextSibling);
        }
        nextSibling = node;
      }
    }

    // Update state
    nodeMap.clear();
    newNodeMap.forEach((v, k) => nodeMap.set(k, v));
    currentKeys = newKeys;
  });
}

export function renderConditional(
  parent: Node,
  anchor: Node,
  condition: () => unknown,
  consequent: () => Node | Node[],
  alternate?: () => Node | Node[]
) {
  let currentNodes: Node[] = [];

  run(() => {
    const shouldShow = !!condition();

    // Remove all current nodes
    for (const node of currentNodes) {
      if (node.parentNode) {
        node.parentNode.removeChild(node);
      }
    }
    currentNodes = [];

    // Get new node(s) based on condition
    let result: Node | Node[] | null = null;
    if (shouldShow) {
      result = consequent();
    } else if (alternate) {
      result = alternate();
    }

    // Handle result (single node or array from .map())
    if (result) {
      if (Array.isArray(result)) {
        for (const node of result) {
          parent.insertBefore(node, anchor);
          currentNodes.push(node);
        }
      } else {
        parent.insertBefore(result, anchor);
        currentNodes.push(result);
      }
    }
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

export { Suspense, getCurrentSuspense, query, lazy } from "./suspense";
export type { SuspenseContext, SuspenseProps, Query } from "./suspense";

// DevTools exports
export {
  enableDevTools,
  isDevToolsEnabled,
  getDevToolsInstance,
} from "./devtools";
export type {
  DevToolsOptions,
  FlickDevTools,
  AnimationSpeed,
  TimelineEntry,
  RenderStats,
  GraphJSON,
} from "./devtools";
