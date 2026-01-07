/**
 * Flick DevTools - Dependency Graph
 *
 * Tracks the reactive dependency graph between signals (fx) and effects (run).
 * Used for visualization and debugging of the reactive system.
 */

/*
 * Type Definitions
 */

export interface GraphNode {
  id: number;
  type: "signal" | "effect";
  name?: string;
  createdAt: number;
}

export interface GraphEdge {
  from: number; // signal id
  to: number; // effect id
}

export interface GraphJSON {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/*
 * Dependency Graph Implementation
 */

class DependencyGraph {
  // All nodes in the graph
  private nodes = new Map<number, GraphNode>();

  // Signal -> Effects that depend on it
  private signalToEffects = new Map<number, Set<number>>();

  // Effect -> Signals it depends on
  private effectToSignals = new Map<number, Set<number>>();

  // Listeners for graph changes
  private listeners = new Set<() => void>();

  /*
   * Add a signal node
   */

  addSignal(id: number, name?: string): void {
    if (this.nodes.has(id)) return;

    this.nodes.set(id, {
      id,
      type: "signal",
      name,
      createdAt: performance.now(),
    });

    this.signalToEffects.set(id, new Set());
    this.notifyListeners();
  }

  addEffect(id: number, name?: string): void {
    if (this.nodes.has(id)) return;

    this.nodes.set(id, {
      id,
      type: "effect",
      name,
      createdAt: performance.now(),
    });

    this.effectToSignals.set(id, new Set());
    this.notifyListeners();
  }

  /*
   * Add a dependency edge: signal is read by effect
   */

  /**
   * Add a dependency edge: signal is read by effect
   */
  addEdge(signalId: number, effectId: number): void {
    // Ensure signal->effects set exists
    let effects = this.signalToEffects.get(signalId);
    if (!effects) {
      effects = new Set();
      this.signalToEffects.set(signalId, effects);
    }
    effects.add(effectId);

    // Ensure effect->signals set exists
    let signals = this.effectToSignals.get(effectId);
    if (!signals) {
      signals = new Set();
      this.effectToSignals.set(effectId, signals);
    }
    signals.add(signalId);

    this.notifyListeners();
  }

  /**
   * Remove all edges for an effect (called when re-running to rebuild deps)
   */
  clearEffectDependencies(effectId: number): void {
    const signals = this.effectToSignals.get(effectId);
    if (signals) {
      // Remove this effect from all its signal's subscriber lists
      for (const signalId of signals) {
        this.signalToEffects.get(signalId)?.delete(effectId);
      }
      signals.clear();
    }
  }

  /**
   * Get all effects that depend on a signal
   */
  getSubscribers(signalId: number): number[] {
    return Array.from(this.signalToEffects.get(signalId) || []);
  }

  /**
   * Get all signals an effect depends on
   */
  getDependencies(effectId: number): number[] {
    return Array.from(this.effectToSignals.get(effectId) || []);
  }

  /**
   * Get a node by ID
   */
  getNode(id: number): GraphNode | undefined {
    return this.nodes.get(id);
  }

  /**
   * Get all signals
   */
  getSignals(): GraphNode[] {
    return Array.from(this.nodes.values()).filter((n) => n.type === "signal");
  }

  /**
   * Get all effects
   */
  getEffects(): GraphNode[] {
    return Array.from(this.nodes.values()).filter((n) => n.type === "effect");
  }

  /**
   * Get graph statistics
   */
  getStats(): {
    signalCount: number;
    effectCount: number;
    edgeCount: number;
    avgDepsPerEffect: number;
    avgSubscribersPerSignal: number;
  } {
    const signals = this.getSignals();
    const effects = this.getEffects();

    let totalEdges = 0;
    for (const effectSet of this.signalToEffects.values()) {
      totalEdges += effectSet.size;
    }

    const avgDepsPerEffect =
      effects.length > 0
        ? Array.from(this.effectToSignals.values()).reduce(
            (sum, set) => sum + set.size,
            0
          ) / effects.length
        : 0;

    const avgSubscribersPerSignal =
      signals.length > 0
        ? Array.from(this.signalToEffects.values()).reduce(
            (sum, set) => sum + set.size,
            0
          ) / signals.length
        : 0;

    return {
      signalCount: signals.length,
      effectCount: effects.length,
      edgeCount: totalEdges,
      avgDepsPerEffect: Math.round(avgDepsPerEffect * 100) / 100,
      avgSubscribersPerSignal: Math.round(avgSubscribersPerSignal * 100) / 100,
    };
  }

  /**
   * Export the graph as JSON for visualization
   */
  toJSON(): GraphJSON {
    const nodes = Array.from(this.nodes.values());
    const edges: GraphEdge[] = [];

    for (const [signalId, effectIds] of this.signalToEffects) {
      for (const effectId of effectIds) {
        edges.push({ from: signalId, to: effectId });
      }
    }

    return { nodes, edges };
  }

  /*
   * Subscribe to graph changes
   */

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notifyListeners(): void {
    this.listeners.forEach((fn) => fn());
  }

  /*
   * Clear the graph
   */

  clear(): void {
    this.nodes.clear();
    this.signalToEffects.clear();
    this.effectToSignals.clear();
    this.notifyListeners();
  }
}

export const depGraph = new DependencyGraph();
