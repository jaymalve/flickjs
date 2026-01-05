/**
 * Flick DevTools - Metrics Store
 *
 * Central store for tracking signal updates, effect executions,
 * and DOM mutation statistics.
 */

/*
 * Type Definitions
 */

export interface SignalUpdateEvent {
  fxId: number;
  name?: string;
  prevValue: unknown;
  nextValue: unknown;
  timestamp: number;
}

export interface EffectExecutionEvent {
  runId: number;
  name?: string;
  duration: number;
  timestamp: number;
  domNodesAffected: number;
  dependencies: number[];
}

export interface RenderStats {
  nodeId: string;
  renderCount: number;
  totalTime: number;
  avgTime: number;
  lastRenderTime: number;
  lastTriggeredBy: string;
}

export interface TimelineEntry {
  type: "signal" | "effect" | "dom";
  id: number;
  name?: string;
  timestamp: number;
  duration?: number;
  details: Record<string, unknown>;
}

export interface DOMStats {
  createElement: number;
  insertBefore: number;
  removeChild: number;
  textContent: number;
  setAttribute: number;
}

export interface ListStats {
  lisComputations: number;
  totalMoves: number;
  totalInserts: number;
  totalDeletes: number;
}

/*
 * Metrics Store Implementation
 */

class MetricsStore {
  // Circular buffer for timeline (limit memory usage)
  private timeline: TimelineEntry[] = [];
  private maxTimelineEntries = 1000;

  // Stats per DOM node
  private nodeStats = new Map<Node, RenderStats>();

  // Global counters
  private domStats: DOMStats = {
    createElement: 0,
    insertBefore: 0,
    removeChild: 0,
    textContent: 0,
    setAttribute: 0,
  };

  // List rendering stats
  private listStats: ListStats = {
    lisComputations: 0,
    totalMoves: 0,
    totalInserts: 0,
    totalDeletes: 0,
  };

  // Pause state
  private _paused = false;

  // Listeners for real-time updates
  private listeners = new Set<() => void>();

  /*
   * Getters / Setters
   */

  get paused(): boolean {
    return this._paused;
  }

  set paused(v: boolean) {
    this._paused = v;
  }

  /*
   * Recording Methods
   */

  recordSignalUpdate(event: SignalUpdateEvent): void {
    if (this._paused) return;

    this.addTimelineEntry({
      type: "signal",
      id: event.fxId,
      name: event.name,
      timestamp: event.timestamp,
      details: {
        prevValue: event.prevValue,
        nextValue: event.nextValue,
      },
    });

    this.notifyListeners();
  }

  recordEffectExecution(event: EffectExecutionEvent): void {
    if (this._paused) return;

    this.addTimelineEntry({
      type: "effect",
      id: event.runId,
      name: event.name,
      timestamp: event.timestamp,
      duration: event.duration,
      details: {
        domNodesAffected: event.domNodesAffected,
        dependencies: event.dependencies,
      },
    });

    this.notifyListeners();
  }

  recordDOMNodeUpdate(
    node: Node,
    _runId: number,
    duration: number,
    triggeredBy?: string
  ): void {
    if (this._paused) return;

    let stats = this.nodeStats.get(node);
    if (!stats) {
      stats = {
        nodeId: this.getNodeIdentifier(node),
        renderCount: 0,
        totalTime: 0,
        avgTime: 0,
        lastRenderTime: 0,
        lastTriggeredBy: "",
      };
      this.nodeStats.set(node, stats);
    }

    stats.renderCount++;
    stats.totalTime += duration;
    stats.avgTime = stats.totalTime / stats.renderCount;
    stats.lastRenderTime = duration;
    if (triggeredBy) stats.lastTriggeredBy = triggeredBy;

    this.notifyListeners();
  }

  incrementDOMStat(stat: keyof DOMStats): void {
    if (this._paused) return;
    this.domStats[stat]++;
  }

  recordListOperation(operation: {
    type: "lis";
    inputSize: number;
    lisSize: number;
    movesRequired: number;
  }): void {
    if (this._paused) return;

    this.listStats.lisComputations++;
    this.listStats.totalMoves += operation.movesRequired;

    this.notifyListeners();
  }

  /*
   * Query Methods
   */

  getTopRenderingNodes(limit = 10): RenderStats[] {
    return Array.from(this.nodeStats.values())
      .sort((a, b) => b.renderCount - a.renderCount)
      .slice(0, limit);
  }

  getTimeline(since?: number): TimelineEntry[] {
    if (since) {
      return this.timeline.filter((e) => e.timestamp >= since);
    }
    return [...this.timeline];
  }

  getRecentTimeline(count = 50): TimelineEntry[] {
    return this.timeline.slice(-count);
  }

  getDOMStats(): DOMStats {
    return { ...this.domStats };
  }

  getListStats(): ListStats {
    return { ...this.listStats };
  }

  getSummary(): {
    totalSignalUpdates: number;
    totalEffectExecutions: number;
    totalDOMNodes: number;
  } {
    const signals = this.timeline.filter((e) => e.type === "signal").length;
    const effects = this.timeline.filter((e) => e.type === "effect").length;

    return {
      totalSignalUpdates: signals,
      totalEffectExecutions: effects,
      totalDOMNodes: this.nodeStats.size,
    };
  }

  /*
   * Subscribe to metrics changes
   */

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notifyListeners(): void {
    this.listeners.forEach((fn) => fn());
  }

  /*
   * Clear the metrics store
   */

  clear(): void {
    this.timeline = [];
    this.nodeStats.clear();
    this.domStats = {
      createElement: 0,
      insertBefore: 0,
      removeChild: 0,
      textContent: 0,
      setAttribute: 0,
    };
    this.listStats = {
      lisComputations: 0,
      totalMoves: 0,
      totalInserts: 0,
      totalDeletes: 0,
    };
    this.notifyListeners();
  }

  /*
   * Private Helpers
   */

  private addTimelineEntry(entry: TimelineEntry): void {
    this.timeline.push(entry);
    if (this.timeline.length > this.maxTimelineEntries) {
      this.timeline.shift();
    }
  }

  private getNodeIdentifier(node: Node): string {
    if (node instanceof Element) {
      const tag = node.tagName.toLowerCase();
      const id = node.id ? `#${node.id}` : "";
      const classes = node.className
        ? `.${String(node.className).split(" ").filter(Boolean).join(".")}`
        : "";
      return `${tag}${id}${classes}`;
    }
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent?.slice(0, 20) || "";
      return `text("${text}${text.length >= 20 ? "..." : ""}")`;
    }
    return node.nodeName;
  }
}

/*
 * Singleton Export
 */

export const metricsStore = new MetricsStore();
