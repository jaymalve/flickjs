/**
 * Flick DevTools - Canvas Overlay
 *
 * Full-viewport canvas for drawing performance overlays
 * with crisp rendering on HiDPI displays.
 */

import { applyOpacity } from "./animations";

/*
 * Types
 */

export interface OverlayDrawData {
  rect: DOMRect;
  opacity: number;
  color: string;
  renderCount: number;
  lastDuration: number;
  signalName?: string;
}

export interface LabelConfig {
  showName: boolean;
  showCount: boolean;
  showDuration: boolean;
}

/*
 * Canvas Overlay Class
 */

export class CanvasOverlay {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private dpr: number = 1;

  // Label configuration
  private labelConfig: LabelConfig = {
    showName: true,
    showCount: true,
    showDuration: true,
  };

  /*
   * Lifecycle
   */

  /**
   * Create and attach the canvas to the DOM
   */
  attach(): void {
    if (this.canvas) return;

    this.canvas = document.createElement("canvas");
    this.canvas.id = "flick-devtools-overlay";

    // Style the canvas as a fixed, non-interactive overlay
    Object.assign(this.canvas.style, {
      position: "fixed",
      top: "0",
      left: "0",
      pointerEvents: "none",
      zIndex: "999999",
    });

    // Append to body first
    document.body.appendChild(this.canvas);

    // Get 2D context with alpha
    this.ctx = this.canvas.getContext("2d", { alpha: true });

    // Now update size (which applies DPR scaling to context)
    this.updateCanvasSize();

    // Handle window resize
    window.addEventListener("resize", this.handleResize);
  }

  /**
   * Remove the canvas from the DOM
   */
  detach(): void {
    if (this.canvas) {
      window.removeEventListener("resize", this.handleResize);
      this.canvas.remove();
      this.canvas = null;
      this.ctx = null;
    }
  }

  /**
   * Check if canvas is attached
   */
  isAttached(): boolean {
    return this.canvas !== null;
  }

  /*
   * Configuration
   */

  /**
   * Update label display settings
   */
  setLabelConfig(config: Partial<LabelConfig>): void {
    this.labelConfig = { ...this.labelConfig, ...config };
  }

  /*
   * Drawing Methods
   */

  /**
   * Clear the entire canvas
   */
  clear(): void {
    if (!this.ctx || !this.canvas) return;
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
  }

  /**
   * Draw a single overlay (border + label)
   */
  drawOverlay(data: OverlayDrawData): void {
    if (!this.ctx) return;

    const { rect, opacity, color, renderCount, lastDuration, signalName } =
      data;

    // Skip if fully transparent
    if (opacity <= 0) return;

    // Skip if element is not visible
    if (rect.width === 0 || rect.height === 0) return;

    const colorWithOpacity = applyOpacity(color, opacity);

    // Draw border
    this.drawBorder(rect, colorWithOpacity);

    // Draw label
    this.drawLabel(rect, opacity, renderCount, lastDuration, signalName);
  }

  /**
   * Draw the border around an element
   */
  private drawBorder(rect: DOMRect, color: string): void {
    if (!this.ctx) return;

    const ctx = this.ctx;
    const lineWidth = 2;

    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;

    // Draw slightly inset to avoid being clipped
    ctx.strokeRect(
      rect.x + lineWidth / 2,
      rect.y + lineWidth / 2,
      rect.width - lineWidth,
      rect.height - lineWidth
    );
  }

  /**
   * Draw the label with render info
   */
  private drawLabel(
    rect: DOMRect,
    opacity: number,
    renderCount: number,
    duration: number,
    signalName?: string
  ): void {
    if (!this.ctx) return;

    const ctx = this.ctx;
    const labelText = this.buildLabelText(renderCount, duration, signalName);

    if (!labelText) return;

    // Font settings
    const fontSize = 11;
    const fontFamily = "monospace";
    ctx.font = `${fontSize}px ${fontFamily}`;

    // Measure text
    const metrics = ctx.measureText(labelText);
    const padding = 4;
    const labelHeight = fontSize + padding * 2;
    const labelWidth = metrics.width + padding * 2;

    // Position label at top-right of element, above it
    let labelX = rect.x + rect.width - labelWidth;
    let labelY = rect.y - labelHeight - 2;

    // Clamp to viewport bounds
    if (labelX < 0) labelX = rect.x;
    if (labelY < 0) labelY = rect.y + 2;
    if (labelX + labelWidth > window.innerWidth) {
      labelX = window.innerWidth - labelWidth - 2;
    }

    // Draw background
    ctx.fillStyle = applyOpacity("rgba(0, 0, 0, 0.85)", opacity);
    ctx.beginPath();
    this.roundRect(ctx, labelX, labelY, labelWidth, labelHeight, 3);
    ctx.fill();

    // Draw text
    ctx.fillStyle = applyOpacity("rgba(255, 255, 255, 1)", opacity);
    ctx.fillText(labelText, labelX + padding, labelY + fontSize + padding - 2);
  }

  /**
   * Build the label text string
   */
  private buildLabelText(
    renderCount: number,
    duration: number,
    signalName?: string
  ): string {
    const parts: string[] = [];

    if (this.labelConfig.showName && signalName) {
      // Truncate long names
      const name = signalName.length > 15
        ? signalName.slice(0, 12) + "..."
        : signalName;
      parts.push(name);
    }

    if (this.labelConfig.showCount) {
      parts.push(`x${renderCount}`);
    }

    if (this.labelConfig.showDuration) {
      // Format duration nicely
      const durationStr =
        duration < 1
          ? `${(duration * 1000).toFixed(0)}μs`
          : `${duration.toFixed(1)}ms`;
      parts.push(durationStr);
    }

    return parts.join(" · ");
  }

  /**
   * Draw a rounded rectangle path
   */
  private roundRect(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    width: number,
    height: number,
    radius: number
  ): void {
    ctx.moveTo(x + radius, y);
    ctx.lineTo(x + width - radius, y);
    ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
    ctx.lineTo(x + width, y + height - radius);
    ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
    ctx.lineTo(x + radius, y + height);
    ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
    ctx.lineTo(x, y + radius);
    ctx.quadraticCurveTo(x, y, x + radius, y);
    ctx.closePath();
  }

  /*
   * Size Management
   */

  /**
   * Update canvas size to match viewport (with DPR scaling)
   */
  private updateCanvasSize(): void {
    if (!this.canvas) return;

    this.dpr = window.devicePixelRatio || 1;

    const width = window.innerWidth;
    const height = window.innerHeight;

    // Set actual canvas size (scaled for DPR)
    this.canvas.width = width * this.dpr;
    this.canvas.height = height * this.dpr;

    // Set display size
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;

    // Scale context to match DPR
    if (this.ctx) {
      this.ctx.scale(this.dpr, this.dpr);
    }
  }

  /**
   * Handle window resize
   */
  private handleResize = (): void => {
    this.updateCanvasSize();
  };
}
