/**
 * Flick DevTools - Public API
 *
 * Performance visualization tool for Flick's fine-grained reactivity system.
 * Inspired by React Scan but tailored for signal-based reactivity.
 *
 * @example
 * ```ts
 * import { enableDevTools } from '@flickjs/runtime';
 *
 * const devtools = enableDevTools({
 *   overlay: true,
 *   toolbar: true,
 *   animationSpeed: 'fast',
 * });
 *
 * // Later...
 * devtools.pause();
 * devtools.resume();
 * devtools.clear();
 * ```
 */

import { metricsStore, type TimelineEntry, type RenderStats } from "./metrics";
import { depGraph, type GraphJSON } from "./dependency-graph";
import {
  instrumentedFx,
  instrumentedRun,
  setInstrumentationCallbacks,
  type Fx,
} from "./instrumentation";

/*
 * Type Definitions
 */

export type AnimationSpeed = "off" | "fast" | "slow";

export interface DevToolsOptions {
  /** Show visual overlays around updated elements. Default: true */
  overlay?: boolean;
  /** Show floating toolbar panel. Default: true */
  toolbar?: boolean;
  /** Console logging of updates. Default: false */
  logToConsole?: boolean;
  /** Animation speed for overlays. Default: 'fast' */
  animationSpeed?: AnimationSpeed;
}

export interface FlickDevTools {
  /** Pause metrics collection */
  pause(): void;
  /** Resume metrics collection */
  resume(): void;
  /** Clear all collected metrics */
  clear(): void;
  /** Get the timeline of events */
  getTimeline(): TimelineEntry[];
  /** Get render stats for top nodes */
  getStats(): RenderStats[];
  /** Get the dependency graph */
  getDependencyGraph(): GraphJSON;
  /** Check if paused */
  isPaused(): boolean;
  /** Destroy devtools and clean up */
  destroy(): void;
}

/*
 * Global State
 */

let devtoolsEnabled = false;
let devtoolsInstance: FlickDevTools | null = null;
let originalFx: typeof import("../index").fx | null = null;
let originalRun: typeof import("../index").run | null = null;

/*
 * Main API
 */

/**
 * Enable Flick DevTools
 *
 * This function activates performance monitoring and visualization.
 * Call this early in your app's initialization.
 *
 * @param options Configuration options
 * @returns DevTools instance with control methods
 */
export function enableDevTools(options: DevToolsOptions = {}): FlickDevTools {
  // Return existing instance if already enabled
  if (devtoolsInstance) {
    console.warn(
      "[Flick DevTools] Already enabled. Returning existing instance."
    );
    return devtoolsInstance;
  }

  const config: Required<DevToolsOptions> = {
    overlay: options.overlay ?? true,
    toolbar: options.toolbar ?? true,
    logToConsole: options.logToConsole ?? false,
    animationSpeed: options.animationSpeed ?? "fast",
  };

  console.log("[Flick DevTools] Initializing...", config);

  devtoolsEnabled = true;

  // Set up instrumentation callbacks
  if (config.logToConsole) {
    setInstrumentationCallbacks({
      onEffectStart: (runId, name) => {
        console.log(`[Flick] Effect start: ${name || `#${runId}`}`);
      },
      onEffectEnd: (runId, duration, domNodes, name) => {
        console.log(
          `[Flick] Effect end: ${name || `#${runId}`} - ${duration.toFixed(
            2
          )}ms, ${domNodes.size} DOM nodes`
        );
      },
      onSignalUpdate: (fxId, name) => {
        console.log(`[Flick] Signal update: ${name || `#${fxId}`}`);
      },
    });
  }

  // TODO: Initialize overlay system (Phase 2)
  // if (config.overlay) {
  //   overlayManager.initialize(config.animationSpeed);
  // }

  // TODO: Initialize toolbar (Phase 3)
  // if (config.toolbar) {
  //   toolbar.attach();
  // }

  // Create the devtools instance
  devtoolsInstance = {
    pause() {
      metricsStore.paused = true;
      console.log("[Flick DevTools] Paused");
    },

    resume() {
      metricsStore.paused = false;
      console.log("[Flick DevTools] Resumed");
    },

    clear() {
      metricsStore.clear();
      depGraph.clear();
      console.log("[Flick DevTools] Cleared");
    },

    getTimeline() {
      return metricsStore.getTimeline();
    },

    getStats() {
      return metricsStore.getTopRenderingNodes(100);
    },

    getDependencyGraph() {
      return depGraph.toJSON();
    },

    isPaused() {
      return metricsStore.paused;
    },

    destroy() {
      // TODO: Clean up overlay and toolbar
      // overlayManager.destroy();
      // toolbar.detach();

      metricsStore.clear();
      depGraph.clear();
      devtoolsEnabled = false;
      devtoolsInstance = null;
      console.log("[Flick DevTools] Destroyed");
    },
  };

  console.log("[Flick DevTools] Ready!");
  console.log("[Flick DevTools] Use devtools.getTimeline() to see events");
  console.log(
    "[Flick DevTools] Use devtools.getDependencyGraph() to see signal/effect graph"
  );

  return devtoolsInstance;
}

/**
 * Check if DevTools is currently enabled
 */
export function isDevToolsEnabled(): boolean {
  return devtoolsEnabled;
}

/**
 * Get the current DevTools instance (if enabled)
 */
export function getDevToolsInstance(): FlickDevTools | null {
  return devtoolsInstance;
}

/*
 * Instrumented Runtime Exports
 */

/**
 * Instrumented version of fx() that tracks metrics when DevTools is enabled.
 * This should replace the original fx() when DevTools is active.
 */
export const fx = instrumentedFx;

/**
 * Instrumented version of run() that tracks metrics when DevTools is enabled.
 * This should replace the original run() when DevTools is active.
 */
export const run = instrumentedRun;

/*
 * Re-exports
 */

export type { TimelineEntry, RenderStats } from "./metrics";
export type { GraphJSON, GraphNode, GraphEdge } from "./dependency-graph";
export type { Fx, FxMetadata, RunMetadata } from "./instrumentation";
