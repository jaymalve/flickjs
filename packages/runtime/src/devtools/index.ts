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
import {
  overlayManager,
  type AnimationSpeed as OverlayAnimationSpeed,
} from "./overlay";

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
  /** Enable/disable overlays */
  setOverlayEnabled(enabled: boolean): void;
  /** Set overlay animation speed */
  setAnimationSpeed(speed: AnimationSpeed): void;
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

  // Initialize overlay system
  if (config.overlay) {
    overlayManager.initialize(config.animationSpeed as OverlayAnimationSpeed);
  }

  // Set up instrumentation callbacks
  setInstrumentationCallbacks({
    onEffectStart: config.logToConsole
      ? (runId, name) => {
          console.log(`[Flick] Effect start: ${name || `#${runId}`}`);
        }
      : undefined,

    onEffectEnd: (runId, duration, domNodes, name) => {
      // Log to console if enabled
      if (config.logToConsole) {
        console.log(
          `[Flick] Effect end: ${name || `#${runId}`} - ${duration.toFixed(
            2
          )}ms, ${domNodes.size} DOM nodes`
        );
      }

      // Show overlay for affected DOM nodes
      if (config.overlay && domNodes.size > 0) {
        overlayManager.showUpdates(domNodes, {
          duration,
          signalName: name,
        });
      }
    },

    onSignalUpdate: config.logToConsole
      ? (fxId, name) => {
          console.log(`[Flick] Signal update: ${name || `#${fxId}`}`);
        }
      : undefined,
  });

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
      overlayManager.clearAll();
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

    setOverlayEnabled(enabled: boolean) {
      overlayManager.setEnabled(enabled);
      console.log(`[Flick DevTools] Overlay ${enabled ? "enabled" : "disabled"}`);
    },

    setAnimationSpeed(speed: AnimationSpeed) {
      overlayManager.setAnimationSpeed(speed as OverlayAnimationSpeed);
      console.log(`[Flick DevTools] Animation speed set to: ${speed}`);
    },

    destroy() {
      // Clean up overlay
      overlayManager.destroy();

      // TODO: Clean up toolbar (Phase 3)
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
