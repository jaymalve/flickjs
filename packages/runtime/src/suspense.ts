import { fx, run, Fx } from "./index";

export interface SuspenseContext {
  register: (promise: Promise<any>) => void;
  pending: Fx<number>;
}

const suspenseStack: SuspenseContext[] = [];

export function getCurrentSuspense(): SuspenseContext | undefined {
  return suspenseStack[suspenseStack.length - 1];
}

export interface SuspenseProps {
  fallback: Node | (() => Node);
  children?: Node | (() => Node);
}

export function Suspense(props: SuspenseProps): Node {
  const container = document.createElement("div");
  container.setAttribute("data-suspense", "");

  const pending = fx(0);

  const context: SuspenseContext = {
    pending,
    register(promise: Promise<any>) {
      pending.set((n) => n + 1);
      promise.finally(() => {
        pending.set((n) => n - 1);
      });
    },
  };

  const resolveFallback = (): Node => {
    return typeof props.fallback === "function"
      ? props.fallback()
      : props.fallback;
  };

  const fallbackNode = resolveFallback();

  const childrenWrapper = document.createElement("div");

  const evaluateChildren = (): Node | null => {
    suspenseStack.push(context);
    try {
      return typeof props.children === "function"
        ? props.children()
        : props.children ?? null;
    } catch (thrown) {
      if (thrown instanceof Promise) {
        // Resource threw a promise, already registered, return placeholder
        return document.createComment("suspense-pending");
      }
      throw thrown;
    } finally {
      suspenseStack.pop();
    }
  };

  run(() => {
    if (pending() > 0) {
      return;
    }
    const result = evaluateChildren();
    childrenWrapper.innerHTML = "";
    childrenWrapper.appendChild(result ?? document.createDocumentFragment());
  });

  run(() => {
    const isPending = pending() > 0;
    container.innerHTML = "";
    container.appendChild(isPending ? fallbackNode : childrenWrapper);
  });

  return container;
}

type QueryState = "pending" | "resolved" | "rejected";

export interface Query<T> {
  (): T | undefined;
  loading: () => boolean;
  error: () => Error | undefined;
  latest: () => T | undefined;
  refetch: () => void;
}

type Fetcher<S, T> = (source: S) => Promise<T>;

export function query<T>(fetcher: () => Promise<T>): Query<T>;
export function query<S, T>(
  source: () => S,
  fetcher: Fetcher<S, T>
): Query<T>;

export function query<S, T>(
  sourceOrFetcher: (() => S) | (() => Promise<T>),
  maybeFetcher?: Fetcher<S, T>
): Query<T> {
  const hasSource = maybeFetcher !== undefined;
  const source = hasSource ? (sourceOrFetcher as () => S) : undefined;
  const fetcher = hasSource
    ? maybeFetcher!
    : (sourceOrFetcher as () => Promise<T>);

  const state = fx<QueryState>("pending");
  const value = fx<T | undefined>(undefined);
  const error = fx<Error | undefined>(undefined);
  const latest = fx<T | undefined>(undefined);

  // Track current promise and which Suspense contexts we've registered with
  let currentPromise: Promise<T> | null = null;
  let registeredWith = new Set<SuspenseContext>();

  const load = (sourceValue?: S) => {
    state.set("pending");
    error.set(undefined);

    // Clear registrations for new load
    registeredWith = new Set<SuspenseContext>();

    const promise = hasSource
      ? (fetcher as Fetcher<S, T>)(sourceValue as S)
      : (fetcher as unknown as () => Promise<T>)();

    currentPromise = promise;

    promise
      .then((result) => {
        value.set(result);
        latest.set(result);
        state.set("resolved");
        currentPromise = null;
      })
      .catch((err) => {
        error.set(err instanceof Error ? err : new Error(String(err)));
        state.set("rejected");
        currentPromise = null;
      });
  };

  if (hasSource) {
    run(() => {
      const sourceValue = source!();
      load(sourceValue);
    });
  } else {
    load();
  }
  // Create the query accessor - throws promise while pending
  const read = (() => {
    if (state() === "pending" && currentPromise) {
      const suspenseContext = getCurrentSuspense();
      if (suspenseContext && !registeredWith.has(suspenseContext)) {
        registeredWith.add(suspenseContext);
        suspenseContext.register(currentPromise);
      }
      throw currentPromise;
    }
    return value();
  }) as Query<T>;

  read.loading = () => state() === "pending";
  read.error = () => error();
  read.latest = () => latest();
  read.refetch = () => {
    if (hasSource) {
      load(source!());
    } else {
      load();
    }
  };

  return read;
}

type LazyComponent<P> = (props: P) => Node;
type LazyLoader<P> = () => Promise<{ default: LazyComponent<P> }>;

export function lazy<P extends Record<string, any> = {}>(
  loader: LazyLoader<P>
): LazyComponent<P> {
  let cachedComponent: LazyComponent<P> | null = null;
  let loadPromise: Promise<void> | null = null;

  return (props: P): Node => {
    if (cachedComponent) {
      return cachedComponent(props);
    }

    const placeholder = document.createComment("lazy");

    const suspenseContext = getCurrentSuspense();

    if (!loadPromise) {
      loadPromise = loader().then((module) => {
        cachedComponent = module.default;
      });
    }

    if (suspenseContext) {
      suspenseContext.register(loadPromise);
    }

    loadPromise.then(() => {
      if (cachedComponent && placeholder.parentNode) {
        const componentNode = cachedComponent(props);
        placeholder.parentNode.replaceChild(componentNode, placeholder);
      }
    });

    return placeholder;
  };
}
