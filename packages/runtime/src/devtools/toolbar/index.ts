/**
 * Flick DevTools - Floating Toolbar Panel
 *
 * A draggable floating panel that displays:
 * - Real-time render stats
 * - Timeline of events
 * - Dependency graph visualization
 * - Controls for pause/resume/clear
 */

import { metricsStore } from "../metrics";
import { depGraph } from "../dependency-graph";
import { overlayManager, type AnimationSpeed } from "../overlay";

/*
 * Types
 */

export interface ToolbarConfig {
  position: { x: number; y: number };
  collapsed: boolean;
  activeTab: "stats" | "timeline" | "graph";
}

/*
 * Styles
 */

const TOOLBAR_STYLES = `
  #flick-devtools-toolbar {
    position: fixed;
    width: 320px;
    background: #1a1a2e;
    border: 1px solid #3a3a5c;
    border-radius: 8px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 12px;
    color: #e0e0e0;
    z-index: 999998;
    overflow: hidden;
    user-select: none;
  }

  #flick-devtools-toolbar.collapsed .flick-devtools-body {
    display: none;
  }

  .flick-devtools-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: linear-gradient(135deg, #16162a 0%, #1e1e3a 100%);
    cursor: move;
    border-bottom: 1px solid #3a3a5c;
  }

  .flick-devtools-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-size: 13px;
  }

  .flick-devtools-logo {
    width: 22px;
    height: 22px;
    background: linear-gradient(135deg, #9333ea 0%, #ec4899 100%);
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 12px;
    color: white;
  }

  .flick-devtools-header-actions {
    display: flex;
    gap: 6px;
  }

  .flick-devtools-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #a0a0c0;
    padding: 5px 8px;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .flick-devtools-btn:hover {
    background: rgba(147, 51, 234, 0.2);
    border-color: rgba(147, 51, 234, 0.4);
    color: #fff;
  }

  .flick-devtools-btn.active {
    background: rgba(147, 51, 234, 0.3);
    border-color: #9333ea;
    color: #fff;
  }

  .flick-devtools-tabs {
    display: flex;
    background: #16162a;
    border-bottom: 1px solid #3a3a5c;
  }

  .flick-devtools-tab {
    flex: 1;
    padding: 10px 8px;
    background: transparent;
    border: none;
    color: #808090;
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    transition: all 0.15s ease;
    border-bottom: 2px solid transparent;
  }

  .flick-devtools-tab:hover {
    color: #c0c0d0;
    background: rgba(255, 255, 255, 0.02);
  }

  .flick-devtools-tab.active {
    color: #9333ea;
    border-bottom-color: #9333ea;
    background: rgba(147, 51, 234, 0.05);
  }

  .flick-devtools-content {
    height: 280px;
    overflow-y: auto;
    background: #1a1a2e;
  }

  .flick-devtools-content::-webkit-scrollbar {
    width: 6px;
  }

  .flick-devtools-content::-webkit-scrollbar-track {
    background: #1a1a2e;
  }

  .flick-devtools-content::-webkit-scrollbar-thumb {
    background: #3a3a5c;
    border-radius: 3px;
  }

  .flick-devtools-panel {
    padding: 12px;
    display: none;
  }

  .flick-devtools-panel.active {
    display: block;
  }

  .flick-devtools-stats-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }

  .flick-devtools-stats-table th,
  .flick-devtools-stats-table td {
    padding: 8px 6px;
    text-align: left;
    border-bottom: 1px solid #2a2a4e;
  }

  .flick-devtools-stats-table th {
    color: #808090;
    font-weight: 500;
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.5px;
  }

  .flick-devtools-stats-table tr:hover td {
    background: rgba(147, 51, 234, 0.05);
  }

  .flick-devtools-stats-table .node-id {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #c0c0e0;
  }

  .flick-devtools-stats-table .count {
    color: #9333ea;
    font-weight: 600;
  }

  .flick-devtools-stats-table .time {
    font-family: monospace;
  }

  .flick-devtools-stats-table .time.fast {
    color: #22c55e;
  }

  .flick-devtools-stats-table .time.medium {
    color: #eab308;
  }

  .flick-devtools-stats-table .time.slow {
    color: #ef4444;
  }

  .flick-devtools-timeline {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .flick-devtools-timeline-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 4px;
    font-size: 11px;
  }

  .flick-devtools-timeline-item .type {
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .flick-devtools-timeline-item .type.signal {
    background: rgba(147, 51, 234, 0.2);
    color: #c084fc;
  }

  .flick-devtools-timeline-item .type.effect {
    background: rgba(34, 197, 94, 0.2);
    color: #86efac;
  }

  .flick-devtools-timeline-item .name {
    flex: 1;
    color: #e0e0e0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .flick-devtools-timeline-item .duration {
    color: #808090;
    font-family: monospace;
    font-size: 10px;
  }

  .flick-devtools-graph {
    text-align: center;
    padding: 20px;
  }

  .flick-devtools-graph-stats {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
    margin-bottom: 16px;
  }

  .flick-devtools-graph-stat {
    background: rgba(255, 255, 255, 0.03);
    padding: 12px;
    border-radius: 6px;
    text-align: center;
  }

  .flick-devtools-graph-stat .value {
    font-size: 24px;
    font-weight: 600;
    color: #9333ea;
  }

  .flick-devtools-graph-stat .label {
    font-size: 10px;
    color: #808090;
    text-transform: uppercase;
    margin-top: 4px;
  }

  .flick-devtools-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: #16162a;
    border-top: 1px solid #3a3a5c;
  }

  .flick-devtools-controls {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .flick-devtools-select {
    background: #2a2a4e;
    border: 1px solid #3a3a5c;
    color: #e0e0e0;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .flick-devtools-select:hover {
    border-color: #9333ea;
  }

  .flick-devtools-summary {
    display: flex;
    gap: 16px;
    color: #808090;
    font-size: 11px;
  }

  .flick-devtools-summary span {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .flick-devtools-summary .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .flick-devtools-summary .dot.signals {
    background: #9333ea;
  }

  .flick-devtools-summary .dot.effects {
    background: #22c55e;
  }

  .flick-devtools-empty {
    text-align: center;
    padding: 40px 20px;
    color: #606080;
  }

  .flick-devtools-empty-icon {
    font-size: 32px;
    margin-bottom: 8px;
    opacity: 0.5;
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
  private updateInterval: number | null = null;
  private isPaused = false;

  constructor(config: Partial<ToolbarConfig> = {}) {
    this.config = {
      position: { x: 20, y: 20 },
      collapsed: false,
      activeTab: "stats",
      ...config,
    };
    this.loadPersistedConfig();
  }

  /*
   * Lifecycle
   */

  attach(): void {
    if (this.container) return;

    // Inject styles
    this.injectStyles();

    // Create container
    this.container = document.createElement("div");
    this.container.id = "flick-devtools-toolbar";
    this.container.innerHTML = this.buildHTML();

    // Apply position
    this.container.style.left = `${this.config.position.x}px`;
    this.container.style.top = `${this.config.position.y}px`;

    if (this.config.collapsed) {
      this.container.classList.add("collapsed");
    }

    document.body.appendChild(this.container);

    // Bind events
    this.bindEvents();

    // Start update loop
    this.startUpdateLoop();

    console.log("[Flick DevTools] Toolbar attached");
  }

  detach(): void {
    if (this.container) {
      this.stopUpdateLoop();
      this.container.remove();
      this.container = null;
    }

    // Remove styles
    const styleEl = document.getElementById("flick-devtools-styles");
    if (styleEl) styleEl.remove();

    console.log("[Flick DevTools] Toolbar detached");
  }

  /*
   * HTML Building
   */

  private buildHTML(): string {
    return `
      <div class="flick-devtools-header" data-draggable>
        <div class="flick-devtools-title">
          <div class="flick-devtools-logo">F</div>
          Flick DevTools
        </div>
        <div class="flick-devtools-header-actions">
          <button class="flick-devtools-btn" data-action="toggle-overlay" title="Toggle Overlay">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
              <line x1="3" y1="9" x2="21" y2="9"/>
            </svg>
          </button>
          <button class="flick-devtools-btn" data-action="collapse" title="Collapse">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
          </button>
        </div>
      </div>

      <div class="flick-devtools-body">
        <div class="flick-devtools-tabs">
          <button class="flick-devtools-tab active" data-tab="stats">Stats</button>
          <button class="flick-devtools-tab" data-tab="timeline">Timeline</button>
          <button class="flick-devtools-tab" data-tab="graph">Graph</button>
        </div>

        <div class="flick-devtools-content">
          <div class="flick-devtools-panel active" data-panel="stats">
            <div class="flick-devtools-stats-container"></div>
          </div>
          <div class="flick-devtools-panel" data-panel="timeline">
            <div class="flick-devtools-timeline-container"></div>
          </div>
          <div class="flick-devtools-panel" data-panel="graph">
            <div class="flick-devtools-graph-container"></div>
          </div>
        </div>

        <div class="flick-devtools-footer">
          <div class="flick-devtools-controls">
            <button class="flick-devtools-btn" data-action="pause" title="Pause/Resume">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="pause-icon">
                <rect x="6" y="4" width="4" height="16"/>
                <rect x="14" y="4" width="4" height="16"/>
              </svg>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class="play-icon" style="display:none">
                <polygon points="5,3 19,12 5,21"/>
              </svg>
            </button>
            <button class="flick-devtools-btn" data-action="clear" title="Clear">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"/>
                <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"/>
              </svg>
            </button>
            <select class="flick-devtools-select" data-action="speed">
              <option value="fast">Fast</option>
              <option value="slow">Slow</option>
              <option value="off">Off</option>
            </select>
          </div>
          <div class="flick-devtools-summary">
            <span><span class="dot signals"></span><span data-metric="signals">0</span> signals</span>
            <span><span class="dot effects"></span><span data-metric="effects">0</span> effects</span>
          </div>
        </div>
      </div>
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
    const header = this.container.querySelector("[data-draggable]");
    if (header) {
      header.addEventListener("mousedown", this.onDragStart as EventListener);
    }

    document.addEventListener("mousemove", this.onDragMove);
    document.addEventListener("mouseup", this.onDragEnd);

    // Button actions
    this.container.addEventListener("click", (e) => {
      const target = e.target as HTMLElement;
      const actionEl = target.closest("[data-action]");
      if (!actionEl) return;

      const action = actionEl.getAttribute("data-action");
      switch (action) {
        case "collapse":
          this.toggleCollapse();
          break;
        case "toggle-overlay":
          this.toggleOverlay();
          break;
        case "pause":
          this.togglePause();
          break;
        case "clear":
          this.clearStats();
          break;
      }
    });

    // Tab switching
    this.container.querySelectorAll("[data-tab]").forEach((tab) => {
      tab.addEventListener("click", () => {
        const tabName = tab.getAttribute("data-tab") as ToolbarConfig["activeTab"];
        this.switchTab(tabName);
      });
    });

    // Speed selector
    const speedSelect = this.container.querySelector(
      '[data-action="speed"]'
    ) as HTMLSelectElement;
    if (speedSelect) {
      speedSelect.addEventListener("change", () => {
        const speed = speedSelect.value as AnimationSpeed;
        overlayManager.setAnimationSpeed(speed);
      });
    }
  }

  private onDragStart = (e: MouseEvent): void => {
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

    const x = Math.max(0, Math.min(e.clientX - this.dragOffset.x, window.innerWidth - 320));
    const y = Math.max(0, Math.min(e.clientY - this.dragOffset.y, window.innerHeight - 100));

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

  private toggleCollapse(): void {
    if (!this.container) return;
    this.config.collapsed = !this.config.collapsed;
    this.container.classList.toggle("collapsed", this.config.collapsed);
    this.persistConfig();
  }

  private toggleOverlay(): void {
    const isEnabled = overlayManager.isEnabled();
    overlayManager.setEnabled(!isEnabled);

    // Update button state
    const btn = this.container?.querySelector('[data-action="toggle-overlay"]');
    if (btn) {
      btn.classList.toggle("active", !isEnabled);
    }
  }

  private togglePause(): void {
    this.isPaused = !this.isPaused;
    metricsStore.paused = this.isPaused;

    // Update button icons
    if (this.container) {
      const pauseIcon = this.container.querySelector(".pause-icon") as HTMLElement;
      const playIcon = this.container.querySelector(".play-icon") as HTMLElement;
      if (pauseIcon && playIcon) {
        pauseIcon.style.display = this.isPaused ? "none" : "block";
        playIcon.style.display = this.isPaused ? "block" : "none";
      }
    }
  }

  private clearStats(): void {
    metricsStore.clear();
    depGraph.clear();
    overlayManager.clearAll();
    this.updateUI();
  }

  private switchTab(tab: ToolbarConfig["activeTab"]): void {
    if (!this.container) return;

    // Update tab buttons
    this.container.querySelectorAll("[data-tab]").forEach((t) => {
      t.classList.toggle("active", t.getAttribute("data-tab") === tab);
    });

    // Update panels
    this.container.querySelectorAll("[data-panel]").forEach((p) => {
      p.classList.toggle("active", p.getAttribute("data-panel") === tab);
    });

    this.config.activeTab = tab;
    this.persistConfig();
    this.updateUI();
  }

  /*
   * Update Loop
   */

  private startUpdateLoop(): void {
    // Update immediately
    this.updateUI();

    // Then update every 500ms
    this.updateInterval = window.setInterval(() => {
      if (!this.isPaused) {
        this.updateUI();
      }
    }, 500);
  }

  private stopUpdateLoop(): void {
    if (this.updateInterval !== null) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
  }

  private updateUI(): void {
    if (!this.container || this.config.collapsed) return;

    this.updateStatsPanel();
    this.updateTimelinePanel();
    this.updateGraphPanel();
    this.updateSummary();
  }

  private updateStatsPanel(): void {
    const container = this.container?.querySelector(".flick-devtools-stats-container");
    if (!container) return;

    const stats = metricsStore.getTopRenderingNodes(15);

    if (stats.length === 0) {
      container.innerHTML = `
        <div class="flick-devtools-empty">
          <div class="flick-devtools-empty-icon">📊</div>
          <div>No render stats yet</div>
          <div style="font-size: 10px; margin-top: 4px;">Interact with your app to see updates</div>
        </div>
      `;
      return;
    }

    container.innerHTML = `
      <table class="flick-devtools-stats-table">
        <thead>
          <tr>
            <th>Element</th>
            <th>Count</th>
            <th>Avg</th>
            <th>Last</th>
          </tr>
        </thead>
        <tbody>
          ${stats
            .map((s) => {
              const avgClass = s.avgTime > 16 ? "slow" : s.avgTime > 4 ? "medium" : "fast";
              const lastClass = s.lastRenderTime > 16 ? "slow" : s.lastRenderTime > 4 ? "medium" : "fast";
              return `
                <tr>
                  <td class="node-id" title="${s.nodeId}">${this.truncate(s.nodeId, 18)}</td>
                  <td class="count">${s.renderCount}</td>
                  <td class="time ${avgClass}">${s.avgTime.toFixed(1)}ms</td>
                  <td class="time ${lastClass}">${s.lastRenderTime.toFixed(1)}ms</td>
                </tr>
              `;
            })
            .join("")}
        </tbody>
      </table>
    `;
  }

  private updateTimelinePanel(): void {
    const container = this.container?.querySelector(".flick-devtools-timeline-container");
    if (!container) return;

    const timeline = metricsStore.getRecentTimeline(30);

    if (timeline.length === 0) {
      container.innerHTML = `
        <div class="flick-devtools-empty">
          <div class="flick-devtools-empty-icon">📜</div>
          <div>No events yet</div>
          <div style="font-size: 10px; margin-top: 4px;">Signal updates and effects will appear here</div>
        </div>
      `;
      return;
    }

    container.innerHTML = `
      <div class="flick-devtools-timeline">
        ${timeline
          .slice()
          .reverse()
          .map((entry) => {
            const name = entry.name || `#${entry.id}`;
            const duration = entry.duration ? `${entry.duration.toFixed(1)}ms` : "";
            return `
              <div class="flick-devtools-timeline-item">
                <span class="type ${entry.type}">${entry.type}</span>
                <span class="name" title="${name}">${this.truncate(name, 20)}</span>
                ${duration ? `<span class="duration">${duration}</span>` : ""}
              </div>
            `;
          })
          .join("")}
      </div>
    `;
  }

  private updateGraphPanel(): void {
    const container = this.container?.querySelector(".flick-devtools-graph-container");
    if (!container) return;

    const stats = depGraph.getStats();

    container.innerHTML = `
      <div class="flick-devtools-graph">
        <div class="flick-devtools-graph-stats">
          <div class="flick-devtools-graph-stat">
            <div class="value">${stats.signalCount}</div>
            <div class="label">Signals</div>
          </div>
          <div class="flick-devtools-graph-stat">
            <div class="value">${stats.effectCount}</div>
            <div class="label">Effects</div>
          </div>
          <div class="flick-devtools-graph-stat">
            <div class="value">${stats.edgeCount}</div>
            <div class="label">Dependencies</div>
          </div>
          <div class="flick-devtools-graph-stat">
            <div class="value">${stats.avgDepsPerEffect}</div>
            <div class="label">Avg Deps/Effect</div>
          </div>
        </div>
        <div style="color: #606080; font-size: 11px;">
          Use <code style="background:#2a2a4e;padding:2px 6px;border-radius:3px;">devtools.getDependencyGraph()</code><br/>
          in console for full graph data
        </div>
      </div>
    `;
  }

  private updateSummary(): void {
    if (!this.container) return;

    const summary = metricsStore.getSummary();

    const signalsEl = this.container.querySelector('[data-metric="signals"]');
    const effectsEl = this.container.querySelector('[data-metric="effects"]');

    if (signalsEl) signalsEl.textContent = String(summary.totalSignalUpdates);
    if (effectsEl) effectsEl.textContent = String(summary.totalEffectExecutions);
  }

  /*
   * Utilities
   */

  private truncate(str: string, len: number): string {
    return str.length > len ? str.slice(0, len - 1) + "…" : str;
  }

  /*
   * Persistence
   */

  private loadPersistedConfig(): void {
    try {
      const saved = localStorage.getItem("flick-devtools-toolbar");
      if (saved) {
        const parsed = JSON.parse(saved);
        this.config = { ...this.config, ...parsed };
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
