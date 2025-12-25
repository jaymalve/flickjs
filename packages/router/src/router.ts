import { run, fx, Suspense } from "@flickjs/runtime";
import { currentPath, params, queryParams } from "./fx";
import { matchRoute, type Route } from "./utils";

/**
 * Update routing state from current URL
 */
function updateRoutingState() {
  const path = window.location.pathname;
  const searchParams = new URLSearchParams(window.location.search);

  currentPath.set(path);
  queryParams.set(searchParams);
}

// Initialize routing state
updateRoutingState();

// Listen to browser back/forward buttons
window.addEventListener("popstate", () => {
  updateRoutingState();
});


function DefaultFallback(): Node {
  const div = document.createElement("div");
  // div.textContent = "Loading...";
  return div;
}

export interface RouterProps {
  routes: Route[];
  fallback?: () => Node;
}

/**
 * Router component that handles route matching and rendering
 * Integrates with Suspense for loading states
 */
export function Router(props: RouterProps) {
  const container = document.createElement("div");
  const currentComponent = fx<{ fn: () => Node } | null>(null);
  const routeError = fx<Error | null>(null);

  // Match route on path change
  run(() => {
    const path = currentPath();
    const match = matchRoute(path, props.routes);

    if (match) {
      // Update params fx
      params.set(match.params);
      routeError.set(null);

      // Load component
      match.route
        .component()
        .then((module) => {
          currentComponent.set({ fn: module.default });
        })
        .catch((error) => {
          console.error("Failed to load route component:", error);
          routeError.set(error);
          currentComponent.set(null);
        });
    } else {
      // No match - render 404
      currentComponent.set({
        fn: () => {
          const div = document.createElement("div");
          div.innerHTML = `<div><h1>404</h1><p>Page not found: ${path}</p></div>`;
          return div.firstElementChild as Node;
        },
      });
    }
  });

  const fallback = props.fallback || DefaultFallback;

  const suspenseContent = Suspense({
    fallback,
    children: () => {
      const component = currentComponent();
      const error = routeError();

      if (error) {
        const errorDiv = document.createElement("div");
        errorDiv.textContent = "Error loading page";
        return errorDiv;
      }

      if (component) {
        try {
          return component.fn();
        } catch (err) {
          console.error("Error rendering component:", err);
          const errorDiv = document.createElement("div");
          errorDiv.textContent = "Error rendering component";
          return errorDiv;
        }
      }

      // Return empty placeholder while waiting for first route match
      return document.createComment("router-placeholder");
    },
  });

  container.appendChild(suspenseContent);

  return container;
}

/**
 * Programmatic navigation
 */
export function navigate(to: string, options?: { replace?: boolean }) {
  const url = new URL(to, window.location.origin);
  const method = options?.replace ? "replaceState" : "pushState";

  window.history[method]({}, "", url.pathname + url.search);
  updateRoutingState();
}

/**
 * Link component that intercepts clicks and uses client-side navigation
 */
export function Link(props: {
  href: string;
  children?: any;
  [key: string]: any;
}) {
  const { href, children, ...rest } = props;
  const anchor = document.createElement("a");
  anchor.href = href;

  // Copy other attributes
  Object.entries(rest).forEach(([key, value]) => {
    if (key === "class") {
      anchor.className = value as string;
    } else if (key.startsWith("on")) {
      // Handle event handlers
      const eventName = key.slice(2).toLowerCase();
      anchor.addEventListener(eventName, value as EventListener);
    } else {
      anchor.setAttribute(key, String(value));
    }
  });

  // Intercept clicks for same-origin navigation
  anchor.addEventListener("click", (e) => {
    const url = new URL(href, window.location.origin);
    if (url.origin === window.location.origin) {
      e.preventDefault();
      navigate(href);
    }
  });

  // Append children - handle various types
  if (children !== undefined) {
    if (typeof children === "string") {
      anchor.textContent = children;
    } else if (children instanceof Node) {
      anchor.appendChild(children);
    } else if (typeof children === "function") {
      // Handle fx-based or function children
      try {
        const childValue = children();
        if (typeof childValue === "string") {
          anchor.textContent = childValue;
        } else if (childValue instanceof Node) {
          anchor.appendChild(childValue);
        } else if (Array.isArray(childValue)) {
          // Handle arrays of children
          childValue.forEach((child) => {
            if (typeof child === "string") {
              anchor.appendChild(document.createTextNode(child));
            } else if (child instanceof Node) {
              anchor.appendChild(child);
            }
          });
        }
      } catch {
        // If calling fails, might be a render function - wrap in run
        run(() => {
          try {
            const childValue = children();
            anchor.innerHTML = "";
            if (typeof childValue === "string") {
              anchor.textContent = childValue;
            } else if (childValue instanceof Node) {
              anchor.appendChild(childValue);
            } else if (Array.isArray(childValue)) {
              childValue.forEach((child) => {
                if (typeof child === "string") {
                  anchor.appendChild(document.createTextNode(child));
                } else if (child instanceof Node) {
                  anchor.appendChild(child);
                }
              });
            }
          } catch (e) {
            // Ignore errors
          }
        });
      }
    } else if (Array.isArray(children)) {
      // Handle arrays of children
      children.forEach((child) => {
        if (typeof child === "string") {
          anchor.appendChild(document.createTextNode(child));
        } else if (child instanceof Node) {
          anchor.appendChild(child);
        }
      });
    }
  }

  return anchor;
}
