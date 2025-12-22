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
