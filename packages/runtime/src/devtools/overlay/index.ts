/**
 * Flick DevTools - Overlay Manager
 *
 * Orchestrates the visual overlay system:
 * - Tracks active overlays for DOM nodes
 * - Runs the render loop
 * - Coordinates canvas and animations
 */

import { CanvasOverlay, type OverlayDrawData } from "./canvas";
import {
  AnimationController,
  type AnimationSpeed,
  getPerformanceColor,
  type OverlayColors,
  DEFAULT_COLORS,
} from "./animations";

/*
 * Types
 */

interface OverlayState {
  /** The DOM node being highlighted */
  node: Node;
  /** Current bounding rect (updated each frame) */
  rect: DOMRect;
  /** Current opacity (calculated from animation) */
  opacity: number;
  /** Border color */
  color: string;
  /** Number of times this node has been updated */
  renderCount: number;
  /** Duration of the last update (ms) */
  lastDuration: number;
  /** Name of the signal that triggered the update */
  signalName?: string;
  /** Time when this overlay was created/refreshed */
  startTime: number;
}

export interface OverlayManagerConfig {
  /** Animation speed for fades */
  animationSpeed: AnimationSpeed;
  /** Whether to show overlays */
  enabled: boolean;
  /** Custom colors */
  colors: OverlayColors;
}

/*
 * Overlay Manager Class
 */

class OverlayManager {
  private canvas: CanvasOverlay;
  private animationController: AnimationController;
  private config: OverlayManagerConfig;

  // Track active overlays by node
  private activeOverlays = new Map<Node, OverlayState>();

  // Render loop state
  private animationFrameId: number | null = null;
  private isRunning = false;

  constructor() {
    this.canvas = new CanvasOverlay();
    this.animationController = new AnimationController("fast");
    this.config = {
      animationSpeed: "fast",
      enabled: true,
      colors: DEFAULT_COLORS,
    };
  }

  /*
   * Lifecycle
   */

  /**
   * Initialize the overlay system
   */
  initialize(speed: AnimationSpeed = "fast"): void {
    this.config.animationSpeed = speed;
    this.animationController.setSpeed(speed);
    this.canvas.attach();
    this.startRenderLoop();

    console.log("[Flick DevTools] Overlay system initialized");
  }

  /**
   * Destroy the overlay system
   */
  destroy(): void {
    this.stopRenderLoop();
    this.canvas.detach();
    this.activeOverlays.clear();

    console.log("[Flick DevTools] Overlay system destroyed");
  }

  /*
   * Configuration
   */

  /**
   * Enable or disable overlays
   */
  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled;

    if (!enabled) {
      this.activeOverlays.clear();
      this.canvas.clear();
    }
  }

  /**
   * Check if overlays are enabled
   */
  isEnabled(): boolean {
    return this.config.enabled;
  }

  /**
   * Set animation speed
   */
  setAnimationSpeed(speed: AnimationSpeed): void {
    this.config.animationSpeed = speed;
    this.animationController.setSpeed(speed);
  }

  /**
   * Get current animation speed
   */
  getAnimationSpeed(): AnimationSpeed {
    return this.config.animationSpeed;
  }

  /**
   * Set custom colors
   */
  setColors(colors: Partial<OverlayColors>): void {
    this.config.colors = { ...this.config.colors, ...colors };
  }

  /*
   * Overlay Management
   */

  /**
   * Show an overlay for a DOM node that was updated
   *
   * Called by the instrumentation layer when an effect updates DOM nodes.
   */
  showUpdate(
    node: Node,
    metadata: {
      duration: number;
      renderCount: number;
      signalName?: string;
    }
  ): void {
    if (!this.config.enabled) return;
    if (!this.animationController.isEnabled()) return;

    const rect = this.getNodeRect(node);
    if (!rect) return;

    // Determine color based on performance
    const color = getPerformanceColor(
      metadata.duration,
      metadata.renderCount,
      this.config.colors
    );

    // Check if we already have an overlay for this node
    const existing = this.activeOverlays.get(node);

    if (existing) {
      // Refresh the existing overlay
      existing.opacity = 1;
      existing.renderCount = metadata.renderCount;
      existing.lastDuration = metadata.duration;
      existing.signalName = metadata.signalName;
      existing.startTime = performance.now();
      existing.color = color;
      existing.rect = rect;
    } else {
      // Create new overlay
      this.activeOverlays.set(node, {
        node,
        rect,
        opacity: 1,
        color,
        renderCount: metadata.renderCount,
        lastDuration: metadata.duration,
        signalName: metadata.signalName,
        startTime: performance.now(),
      });
    }
  }

  /**
   * Show overlays for multiple nodes at once
   */
  showUpdates(
    nodes: Set<Node>,
    metadata: {
      duration: number;
      signalName?: string;
    }
  ): void {
    for (const node of nodes) {
      // Get existing render count or start at 1
      const existing = this.activeOverlays.get(node);
      const renderCount = existing ? existing.renderCount + 1 : 1;

      this.showUpdate(node, {
        duration: metadata.duration,
        renderCount,
        signalName: metadata.signalName,
      });
    }
  }

  /**
   * Clear all overlays
   */
  clearAll(): void {
    this.activeOverlays.clear();
    this.canvas.clear();
  }

  /**
   * Get count of active overlays
   */
  getActiveCount(): number {
    return this.activeOverlays.size;
  }

  /*
   * Render Loop
   */

  /**
   * Start the render loop
   */
  private startRenderLoop(): void {
    if (this.isRunning) return;

    this.isRunning = true;
    this.renderFrame();
  }

  /**
   * Stop the render loop
   */
  private stopRenderLoop(): void {
    this.isRunning = false;

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  /**
   * Render a single frame
   */
  private renderFrame = (): void => {
    if (!this.isRunning) return;

    // Schedule next frame first (for smooth animation)
    this.animationFrameId = requestAnimationFrame(this.renderFrame);

    // Clear canvas
    this.canvas.clear();

    // Skip if disabled
    if (!this.config.enabled) return;

    const now = performance.now();
    const toRemove: Node[] = [];

    // Update and draw each overlay
    for (const [node, state] of this.activeOverlays) {
      // Update rect in case element moved or resized
      const newRect = this.getNodeRect(node);

      if (!newRect) {
        // Node no longer visible, remove it
        toRemove.push(node);
        continue;
      }

      state.rect = newRect;

      // Calculate elapsed time
      const elapsed = now - state.startTime;

      // Check if should be removed
      if (this.animationController.shouldRemove(elapsed)) {
        toRemove.push(node);
        continue;
      }

      // Calculate current opacity
      state.opacity = this.animationController.calculateOpacity(elapsed);

      // Draw the overlay
      const drawData: OverlayDrawData = {
        rect: state.rect,
        opacity: state.opacity,
        color: state.color,
        renderCount: state.renderCount,
        lastDuration: state.lastDuration,
        signalName: state.signalName,
      };

      this.canvas.drawOverlay(drawData);
    }

    // Clean up finished overlays
    for (const node of toRemove) {
      this.activeOverlays.delete(node);
    }
  };

  /*
   * Utilities
   */

  /**
   * Get the bounding rect for a node
   */
  private getNodeRect(node: Node): DOMRect | null {
    // Element nodes
    if (node instanceof Element) {
      const rect = node.getBoundingClientRect();

      // Skip if not visible
      if (rect.width === 0 && rect.height === 0) {
        return null;
      }

      return rect;
    }

    // Text nodes - use parent element
    if (node.nodeType === Node.TEXT_NODE && node.parentElement) {
      const rect = node.parentElement.getBoundingClientRect();

      if (rect.width === 0 && rect.height === 0) {
        return null;
      }

      return rect;
    }

    return null;
  }
}

/*
 * Singleton Export
 */

export const overlayManager = new OverlayManager();

// Re-export types
export type { AnimationSpeed } from "./animations";
export { DEFAULT_COLORS, type OverlayColors } from "./animations";
