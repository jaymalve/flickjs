import { signal, effect } from "./index";

export interface SuspenseContext {
  register: (promise: Promise<any>) => void;
  pending: ReturnType<typeof signal<number>>;
}

const suspenseStack: SuspenseContext[] = [];

export function getCurrentSuspense(): SuspenseContext | undefined {
  return suspenseStack[suspenseStack.length - 1];
}

export interface SuspenseProps {
  fallback: Node | (() => Node);
  children: Node | (() => Node);
}

export function Suspense(props: SuspenseProps): Node {
  const container = document.createElement("div");
  container.setAttribute("data-suspense", "");

  const pending = signal(0);

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

  const resolveChildren = (): Node => {
    suspenseStack.push(context);
    const children =
      typeof props.children === "function" ? props.children() : props.children;
    suspenseStack.pop();
    return children;
  };

  const childrenNode = resolveChildren();

  let currentContent: Node | null = null;

  effect(() => {
    const isPending = pending() > 0;
    const nextContent = isPending ? resolveFallback() : childrenNode;

    if (currentContent !== nextContent) {
      container.innerHTML = "";
      container.appendChild(nextContent);
      currentContent = nextContent;
    }
  });

  return container;
}

type ResourceState = "pending" | "resolved" | "rejected";

export interface Resource<T> {
  (): T | undefined;
  loading: () => boolean;
  error: () => Error | undefined;
  latest: () => T | undefined;
  refetch: () => void;
}

type Fetcher<S, T> = (source: S) => Promise<T>;

export function resource<T>(fetcher: () => Promise<T>): Resource<T>;
export function resource<S, T>(
  source: () => S,
  fetcher: Fetcher<S, T>
): Resource<T>;

export function resource<S, T>(
  sourceOrFetcher: (() => S) | (() => Promise<T>),
  maybeFetcher?: Fetcher<S, T>
): Resource<T> {
  const hasSource = maybeFetcher !== undefined;
  const source = hasSource ? (sourceOrFetcher as () => S) : undefined;
  const fetcher = hasSource
    ? maybeFetcher!
    : (sourceOrFetcher as () => Promise<T>);

  const state = signal<ResourceState>("pending");
  const value = signal<T | undefined>(undefined);
  const error = signal<Error | undefined>(undefined);
  const latest = signal<T | undefined>(undefined);

  const suspenseContext = getCurrentSuspense();

  const load = (sourceValue?: S) => {
    state.set("pending");
    error.set(undefined);

    const promise = hasSource
      ? (fetcher as Fetcher<S, T>)(sourceValue as S)
      : (fetcher as unknown as () => Promise<T>)();

    if (suspenseContext) {
      suspenseContext.register(promise);
    }

    promise
      .then((result) => {
        value.set(result);
        latest.set(result);
        state.set("resolved");
      })
      .catch((err) => {
        error.set(err instanceof Error ? err : new Error(String(err)));
        state.set("rejected");
      });
  };

  if (hasSource) {
    effect(() => {
      const sourceValue = source!();
      load(sourceValue);
    });
  } else {
    load();
  }

  // Create the resource accessor
  const read = (() => value()) as Resource<T>; // Guarantees type safety
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
