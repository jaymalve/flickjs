/**
 * Flick DevTools - Minimal Overlay Toggle Pill
 *
 * A small draggable pill that toggles the overlay on/off.
 */

import { overlayManager } from "../overlay";

/*
 * Types
 */

export interface ToolbarConfig {
  position: { x: number; y: number };
}

/*
 * Styles
 */

const TOOLBAR_STYLES = `
  #flick-devtools-pill {
    position: fixed;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: #1a1a2e;
    border: 1px solid #3a3a5c;
    border-radius: 20px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.4);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 12px;
    color: #e0e0e0;
    z-index: 999998;
    user-select: none;
    cursor: move;
  }

  .flick-pill-logo {
    width: 20px;
    height: 20px;
    background: linear-gradient(135deg, #9333ea 0%, #ec4899 100%);
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 11px;
    color: white;
    flex-shrink: 0;
  }

  .flick-pill-toggle {
    position: relative;
    width: 36px;
    height: 20px;
    background: #2a2a4e;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.2s ease;
  }

  .flick-pill-toggle::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: #606080;
    border-radius: 50%;
    transition: transform 0.2s ease, background 0.2s ease;
  }

  .flick-pill-toggle.active {
    background: rgba(147, 51, 234, 0.3);
  }

  .flick-pill-toggle.active::after {
    transform: translateX(16px);
    background: #9333ea;
  }
`;

/*
 * Toolbar Class
 */

export class DevToolsToolbar {
  private container: HTMLElement | null = null;
  private config: ToolbarConfig;
  private isDragging = false;
  private dragOffset = { x: 0, y: 0 };

  constructor(config: Partial<ToolbarConfig> = {}) {
    this.config = {
      position: { x: 20, y: 20 },
      ...config,
    };
    this.loadPersistedConfig();
  }

  /*
   * Lifecycle
   */

  attach(): void {
    if (this.container) return;

    this.injectStyles();

    this.container = document.createElement("div");
    this.container.id = "flick-devtools-pill";
    this.container.innerHTML = this.buildHTML();

    this.container.style.left = `${this.config.position.x}px`;
    this.container.style.top = `${this.config.position.y}px`;

    document.body.appendChild(this.container);

    this.bindEvents();

    // Sync toggle state with overlay manager
    const toggle = this.container.querySelector(".flick-pill-toggle");
    if (toggle && overlayManager.isEnabled()) {
      toggle.classList.add("active");
    }

    console.log("[Flick DevTools] Pill attached");
  }

  detach(): void {
    if (this.container) {
      this.container.remove();
      this.container = null;
    }

    const styleEl = document.getElementById("flick-devtools-styles");
    if (styleEl) styleEl.remove();

    console.log("[Flick DevTools] Pill detached");
  }

  /*
   * HTML Building
   */

  private buildHTML(): string {
    return `
      <div class="flick-pill-logo">F</div>
      <button class="flick-pill-toggle" data-action="toggle-overlay" title="Toggle Overlay"></button>
    `;
  }

  /*
   * Styles
   */

  private injectStyles(): void {
    if (document.getElementById("flick-devtools-styles")) return;

    const style = document.createElement("style");
    style.id = "flick-devtools-styles";
    style.textContent = TOOLBAR_STYLES;
    document.head.appendChild(style);
  }

  /*
   * Event Handling
   */

  private bindEvents(): void {
    if (!this.container) return;

    // Drag handling
    this.container.addEventListener("mousedown", this.onDragStart as EventListener);
    document.addEventListener("mousemove", this.onDragMove);
    document.addEventListener("mouseup", this.onDragEnd);

    // Toggle overlay
    this.container.addEventListener("click", (e) => {
      const target = e.target as HTMLElement;
      if (target.closest("[data-action='toggle-overlay']")) {
        this.toggleOverlay();
      }
    });
  }

  private onDragStart = (e: MouseEvent): void => {
    // Don't start drag if clicking the toggle button
    if ((e.target as HTMLElement).closest("[data-action]")) return;

    if (!this.container) return;
    this.isDragging = true;
    const rect = this.container.getBoundingClientRect();
    this.dragOffset = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
    this.container.style.transition = "none";
  };

  private onDragMove = (e: MouseEvent): void => {
    if (!this.isDragging || !this.container) return;

    const x = Math.max(0, Math.min(e.clientX - this.dragOffset.x, window.innerWidth - 100));
    const y = Math.max(0, Math.min(e.clientY - this.dragOffset.y, window.innerHeight - 40));

    this.container.style.left = `${x}px`;
    this.container.style.top = `${y}px`;

    this.config.position = { x, y };
  };

  private onDragEnd = (): void => {
    if (this.isDragging) {
      this.isDragging = false;
      this.persistConfig();
      if (this.container) {
        this.container.style.transition = "";
      }
    }
  };

  /*
   * Actions
   */

  private toggleOverlay(): void {
    const isEnabled = overlayManager.isEnabled();
    overlayManager.setEnabled(!isEnabled);

    const toggle = this.container?.querySelector(".flick-pill-toggle");
    if (toggle) {
      toggle.classList.toggle("active", !isEnabled);
    }
  }

  /*
   * Persistence
   */

  private loadPersistedConfig(): void {
    try {
      const saved = localStorage.getItem("flick-devtools-toolbar");
      if (saved) {
        const parsed = JSON.parse(saved);
        if (parsed.position) {
          this.config.position = parsed.position;
        }
      }
    } catch {
      // Ignore errors
    }
  }

  private persistConfig(): void {
    try {
      localStorage.setItem("flick-devtools-toolbar", JSON.stringify(this.config));
    } catch {
      // Ignore errors
    }
  }
}

/*
 * Singleton Export
 */

export const toolbar = new DevToolsToolbar();
