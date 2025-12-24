export interface Route {
  path: string;
  component: () => Promise<{ default: () => Node }>;
}

export interface MatchResult {
  route: Route;
  params: Record<string, string>;
}

/**
 * Convert a file path pattern to a route pattern
 * Examples:
 *   pages/index.tsx -> /
 *   pages/about.tsx -> /about
 *   pages/users/[id].tsx -> /users/:id
 *   pages/posts/[...slug].tsx -> /posts/*slug
 */
export function filePathToRoute(
  filePath: string,
  pagesDir: string = "pages"
): string {
  // Remove pages directory prefix and file extension
  let route = filePath
    .replace(new RegExp(`^${pagesDir}/`), "")
    .replace(/\.[jt]sx?$/, "");

  // Handle index files
  if (route === "index" || route.endsWith("/index")) {
    route = route.replace(/\/?index$/, "") || "/";
  }

  // Convert [param] to :param
  route = route.replace(/\[([^\]]+)\]/g, (_, param) => {
    // Handle catch-all [...slug]
    if (param.startsWith("...")) {
      return `*${param.slice(3)}`;
    }
    return `:${param}`;
  });

  // Ensure leading slash
  if (!route.startsWith("/")) {
    route = "/" + route;
  }

  return route;
}

/**
 * Convert a route pattern to a regex
 * Examples:
 *   / -> /^\/$/
 *   /about -> /^\/about$/
 *   /users/:id -> /^\/users\/([^\/]+)$/
 *   /posts/*slug -> /^\/posts\/(.+)$/
 */
export function pathToRegex(pattern: string): RegExp {
  // Escape special regex characters except : and *
  let regex = pattern
    .replace(/\//g, "\\/")
    .replace(/\*([^\/\*]+)/g, "(.+)") // *param -> (.+) - catch-all with name
    .replace(/\*/g, ".*") // standalone * -> .*
    .replace(/:([^\/\*]+)/g, "([^\\/]+)"); // :param -> ([^\/]+) - single segment param

  return new RegExp(`^${regex}$`);
}

/**
 * Extract parameter names from a route pattern
 */
export function extractParamNames(pattern: string): string[] {
  const params: string[] = [];
  const paramRegex = /[:*]([^\/\*]+)/g;
  let match;

  while ((match = paramRegex.exec(pattern)) !== null) {
    const paramName = match[1];
    // Remove ... prefix for catch-all params
    params.push(paramName.startsWith("...") ? paramName.slice(3) : paramName);
  }

  return params;
}

/**
 * Match a path against routes and return the first match with extracted params
 */
export function matchRoute(path: string, routes: Route[]): MatchResult | null {
  for (const route of routes) {
    const regex = pathToRegex(route.path);
    const match = path.match(regex);

    if (match) {
      const paramNames = extractParamNames(route.path);
      const params: Record<string, string> = {};

      // Extract parameter values from regex match groups
      paramNames.forEach((name, index) => {
        params[name] = match[index + 1] || "";
      });

      return { route, params };
    }
  }

  return null;
}


