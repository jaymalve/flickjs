/**
 * Flick DevTools - Runtime Instrumentation
 *
 * Provides instrumented versions of fx() and run() that collect
 * timing data, track dependencies, and record metrics.
 */

import { metricsStore } from "./metrics";
import { depGraph } from "./dependency-graph";

/*
 * Type Definitions
 */

type Run = () => void;

export type Fx<T> = (() => T) & { set: (next: T | ((v: T) => T)) => void };

export interface FxMetadata {
  id: number;
  name?: string;
  createdAt: number;
  updateCount: number;
  lastUpdateTime: number;
}

export interface RunMetadata {
  id: number;
  name?: string;
  executionCount: number;
  totalTime: number;
  lastExecutionTime: number;
  dependencies: Set<number>;
  domNodes: Set<Node>;
}

export interface InstrumentationCallbacks {
  onEffectStart?: (runId: number, name?: string) => void;
  onEffectEnd?: (
    runId: number,
    duration: number,
    domNodes: Set<Node>,
    name?: string
  ) => void;
  onSignalUpdate?: (fxId: number, name?: string) => void;
}

/*
 * Global State
 */

// Counters for unique IDs
let fxIdCounter = 0;
let runIdCounter = 0;

// Active run context for dependency tracking
let activeRun: Run | null = null;
let activeRunContext: RunMetadata | null = null;

// WeakMaps to attach debug metadata without modifying originals
const fxMetadataMap = new WeakMap<Function, FxMetadata>();
const runMetadataMap = new WeakMap<Function, RunMetadata>();

// Callbacks for overlay integration
let callbacks: InstrumentationCallbacks = {};

/*
 * Configuration
 */

export function setInstrumentationCallbacks(
  newCallbacks: InstrumentationCallbacks
): void {
  callbacks = newCallbacks;
}

/*
 * Metadata Access
 */

export function getFxMetadata(fx: Function): FxMetadata | undefined {
  return fxMetadataMap.get(fx);
}

export function getRunMetadata(run: Function): RunMetadata | undefined {
  return runMetadataMap.get(run);
}

export function getActiveRunContext(): RunMetadata | null {
  return activeRunContext;
}

/*
 * Instrumented fx()
 */

export function instrumentedFx<T>(value: T, name?: string): Fx<T> {
  const subs = new Set<Run>();
  const fxId = fxIdCounter++;

  // Register in dependency graph
  depGraph.addSignal(fxId, name);

  // Track metadata
  const metadata: FxMetadata = {
    id: fxId,
    name,
    createdAt: performance.now(),
    updateCount: 0,
    lastUpdateTime: 0,
  };

  function read(): T {
    // Track dependency: this effect depends on this signal
    if (activeRunContext) {
      activeRunContext.dependencies.add(fxId);
      depGraph.addEdge(fxId, activeRunContext.id);
    }

    // Original subscription logic
    if (activeRun) {
      subs.add(activeRun);
    }

    return value;
  }

  read.set = (next: T | ((v: T) => T)) => {
    const prevValue = value;
    value = typeof next === "function" ? (next as (v: T) => T)(value) : next;

    // Record the signal change
    metadata.updateCount++;
    metadata.lastUpdateTime = performance.now();

    metricsStore.recordSignalUpdate({
      fxId,
      name: metadata.name,
      prevValue,
      nextValue: value,
      timestamp: metadata.lastUpdateTime,
    });

    // Notify callback for overlay
    callbacks.onSignalUpdate?.(fxId, name);

    // Notify all subscribers
    subs.forEach((fn) => fn());
  };

  // Attach metadata
  fxMetadataMap.set(read, metadata);

  return read as Fx<T>;
}

/*
 * Instrumented run()
 */

export function instrumentedRun(fn: Run, name?: string): void {
  const runId = runIdCounter++;

  // Register in dependency graph
  depGraph.addEffect(runId, name);

  const metadata: RunMetadata = {
    id: runId,
    name,
    executionCount: 0,
    totalTime: 0,
    lastExecutionTime: 0,
    dependencies: new Set(),
    domNodes: new Set(),
  };

  const execute = () => {
    const startTime = performance.now();

    // Clear previous dependencies (re-track on each run)
    depGraph.clearEffectDependencies(runId);
    metadata.dependencies.clear();
    metadata.domNodes.clear();

    // Set up DOM mutation tracking
    const mutationObserver = createMutationObserver(metadata);

    // Set active context for dependency tracking
    const prevContext = activeRunContext;
    const prevRun = activeRun;
    activeRunContext = metadata;
    activeRun = execute;

    // Notify callback
    callbacks.onEffectStart?.(runId, name);

    try {
      fn();
    } finally {
      activeRun = prevRun;
      activeRunContext = prevContext;

      // Stop observing mutations
      mutationObserver.disconnect();
    }

    const endTime = performance.now();
    const duration = endTime - startTime;

    // Update metadata
    metadata.executionCount++;
    metadata.totalTime += duration;
    metadata.lastExecutionTime = duration;

    // Record the effect execution
    metricsStore.recordEffectExecution({
      runId,
      name: metadata.name,
      duration,
      timestamp: startTime,
      domNodesAffected: metadata.domNodes.size,
      dependencies: Array.from(metadata.dependencies),
    });

    // Record stats for each affected DOM node
    for (const node of metadata.domNodes) {
      metricsStore.recordDOMNodeUpdate(node, runId, duration, name);
    }

    // Notify callback for overlay
    callbacks.onEffectEnd?.(runId, duration, metadata.domNodes, name);
  };

  // Attach metadata
  runMetadataMap.set(execute, metadata);

  // Execute immediately
  execute();
}

/*
 * DOM Mutation Tracking
 */

function createMutationObserver(metadata: RunMetadata): MutationObserver {
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      // Track the target node
      metadata.domNodes.add(mutation.target);

      // Track added nodes
      mutation.addedNodes.forEach((node) => {
        metadata.domNodes.add(node);
      });

      // Note: We don't track removed nodes in domNodes since they're gone,
      // but we could track removal counts if needed
    }
  });

  // Observe the entire document for mutations during this run
  // Use a try-catch in case document.body doesn't exist yet
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
    // Silently fail if DOM not ready
  }

  return observer;
}

/*
 * Reset (for testing)
 */

export function resetInstrumentation(): void {
  fxIdCounter = 0;
  runIdCounter = 0;
  activeRun = null;
  activeRunContext = null;
  callbacks = {};
}
