/**
 * Flick DevTools - Animation Controller
 *
 * Controls the timing and easing of overlay fade animations.
 */

export type AnimationSpeed = "off" | "fast" | "slow";

/*
 * Animation Configuration
 */

interface AnimationConfig {
  /** Duration of the fade-out in milliseconds */
  fadeDuration: number;
  /** Initial hold time before fade starts (ms) */
  holdDuration: number;
  /** Easing function for the fade */
  easing: (t: number) => number;
}

const ANIMATION_CONFIGS: Record<AnimationSpeed, AnimationConfig> = {
  off: {
    fadeDuration: 0,
    holdDuration: 0,
    easing: () => 0,
  },
  fast: {
    fadeDuration: 400,
    holdDuration: 100,
    easing: easeOutCubic,
  },
  slow: {
    fadeDuration: 1500,
    holdDuration: 500,
    easing: easeOutCubic,
  },
};

/*
 * Easing Functions
 */

/** Smooth deceleration - starts fast, ends slow */
function easeOutCubic(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}

/** Linear - constant speed */
export function linear(t: number): number {
  return t;
}

/** Smooth acceleration and deceleration */
export function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

/*
 * Animation Controller Class
 */

export class AnimationController {
  private speed: AnimationSpeed;
  private config: AnimationConfig;

  constructor(speed: AnimationSpeed = "fast") {
    this.speed = speed;
    this.config = ANIMATION_CONFIGS[speed];
  }

  /**
   * Set the animation speed
   */
  setSpeed(speed: AnimationSpeed): void {
    this.speed = speed;
    this.config = ANIMATION_CONFIGS[speed];
  }

  /**
   * Get current animation speed
   */
  getSpeed(): AnimationSpeed {
    return this.speed;
  }

  /**
   * Get total duration (hold + fade)
   */
  getTotalDuration(): number {
    return this.config.holdDuration + this.config.fadeDuration;
  }

  /**
   * Get just the fade duration
   */
  getFadeDuration(): number {
    return this.config.fadeDuration;
  }

  /**
   * Get the hold duration before fade starts
   */
  getHoldDuration(): number {
    return this.config.holdDuration;
  }

  /**
   * Check if animations are enabled
   */
  isEnabled(): boolean {
    return this.speed !== "off";
  }

  /**
   * Calculate opacity based on elapsed time since overlay appeared
   *
   * @param elapsed - Time in ms since the overlay was shown
   * @returns Opacity value between 0 and 1
   */
  calculateOpacity(elapsed: number): number {
    if (!this.isEnabled()) return 0;

    const { holdDuration, fadeDuration, easing } = this.config;

    // During hold period, full opacity
    if (elapsed < holdDuration) {
      return 1;
    }

    // After hold period, start fading
    const fadeElapsed = elapsed - holdDuration;

    if (fadeElapsed >= fadeDuration) {
      return 0;
    }

    // Calculate progress through fade (0 to 1)
    const progress = fadeElapsed / fadeDuration;

    // Apply easing and invert (we want 1 -> 0)
    return 1 - easing(progress);
  }

  /**
   * Check if an overlay should be removed based on elapsed time
   */
  shouldRemove(elapsed: number): boolean {
    if (!this.isEnabled()) return true;
    return elapsed >= this.getTotalDuration();
  }
}

/*
 * Color Utilities for Performance-Based Coloring
 */

export interface OverlayColors {
  /** Normal updates (< 4ms) */
  fast: string;
  /** Medium updates (4-16ms) */
  medium: string;
  /** Slow updates (> 16ms, frame drop territory) */
  slow: string;
  /** Frequent re-renders (high count) */
  frequent: string;
}

export const DEFAULT_COLORS: OverlayColors = {
  fast: "rgba(34, 197, 94, 0.8)", // Green
  medium: "rgba(147, 51, 234, 0.8)", // Purple (default)
  slow: "rgba(239, 68, 68, 0.8)", // Red
  frequent: "rgba(245, 158, 11, 0.8)", // Orange/Amber
};

/**
 * Determine overlay color based on render performance
 */
export function getPerformanceColor(
  duration: number,
  renderCount: number,
  colors: OverlayColors = DEFAULT_COLORS
): string {
  // Frequent renders take priority (potential performance issue)
  if (renderCount > 10) {
    return colors.frequent;
  }

  // Color based on duration
  if (duration > 16) {
    return colors.slow; // Likely causing frame drops
  }

  if (duration > 4) {
    return colors.medium; // Noticeable but acceptable
  }

  return colors.fast; // Fast update
}

/**
 * Apply opacity to an rgba color string
 */
export function applyOpacity(color: string, opacity: number): string {
  // Handle rgba colors
  if (color.startsWith("rgba")) {
    return color.replace(/,\s*[\d.]+\)$/, `, ${opacity})`);
  }

  // Handle rgb colors - convert to rgba
  if (color.startsWith("rgb(")) {
    return color.replace("rgb(", "rgba(").replace(")", `, ${opacity})`);
  }

  // Handle hex colors
  if (color.startsWith("#")) {
    const hex = color.slice(1);
    const r = parseInt(hex.slice(0, 2), 16);
    const g = parseInt(hex.slice(2, 4), 16);
    const b = parseInt(hex.slice(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${opacity})`;
  }

  return color;
}
