use crate::project::ProjectInfo;
use crate::rules::{self, Severity};
use miette::{miette, Result};
use std::collections::{BTreeSet, HashMap};

const DEAD_CODE_RULES: [(&str, Severity); 3] = [
    ("unused-export", Severity::Warning),
    ("unused-file", Severity::Warning),
    ("unused-dependency", Severity::Warning),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleScope {
    Universal,
    React,
    NextJs,
    ReactNativeOrExpo,
    ServerFramework,
}

impl RuleScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Universal => "Universal",
            Self::React => "React projects",
            Self::NextJs => "Next.js projects",
            Self::ReactNativeOrExpo => "React Native / Expo",
            Self::ServerFramework => "Server framework projects",
        }
    }

    pub fn applies_to_project(self, project: &ProjectInfo) -> bool {
        match self {
            Self::Universal => true,
            Self::React => project.has_react,
            Self::NextJs => project.has_next,
            Self::ReactNativeOrExpo => project.has_react_native || project.has_expo,
            Self::ServerFramework => project.has_server_framework(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleGroup {
    pub key: &'static str,
    pub title: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleCatalogEntry {
    pub id: String,
    pub group_key: &'static str,
    pub scope: RuleScope,
    pub default_severity: Severity,
    pub summary: String,
    pub why: String,
}

impl RuleCatalogEntry {
    pub fn config_snippet(&self) -> String {
        let severity = match self.default_severity {
            Severity::Error => "error",
            Severity::Warning => "warn",
        };
        format!("\"{}\": \"{severity}\"", self.id)
    }

    pub fn disable_snippet(&self) -> String {
        format!("\"{}\": \"off\"", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct RuleCatalog {
    pub groups: Vec<RuleGroup>,
    pub entries: Vec<RuleCatalogEntry>,
}

impl RuleCatalog {
    pub fn group_index(&self, key: &str) -> Option<usize> {
        self.groups.iter().position(|group| group.key == key)
    }
}

#[derive(Debug, Clone, Copy)]
struct RuleSeed {
    id: &'static str,
    group_key: &'static str,
    scope: RuleScope,
}

const GROUPS: [RuleGroup; 15] = [
    RuleGroup {
        key: "core",
        title: "Core JS/TS",
    },
    RuleGroup {
        key: "dead-code",
        title: "Dead Code",
    },
    RuleGroup {
        key: "universal-security",
        title: "Universal Security",
    },
    RuleGroup {
        key: "js-performance",
        title: "JS Performance",
    },
    RuleGroup {
        key: "react-hooks",
        title: "React Hooks",
    },
    RuleGroup {
        key: "react-correctness",
        title: "React Correctness",
    },
    RuleGroup {
        key: "react-architecture",
        title: "React Architecture",
    },
    RuleGroup {
        key: "react-performance",
        title: "React Performance",
    },
    RuleGroup {
        key: "nextjs",
        title: "Next.js",
    },
    RuleGroup {
        key: "server-components",
        title: "Server Components",
    },
    RuleGroup {
        key: "react-native",
        title: "React Native",
    },
    RuleGroup {
        key: "server-security",
        title: "Server Security",
    },
    RuleGroup {
        key: "server-reliability",
        title: "Server Reliability",
    },
    RuleGroup {
        key: "server-performance",
        title: "Server Performance",
    },
    RuleGroup {
        key: "server-architecture",
        title: "Server Architecture",
    },
];

pub fn build_rule_catalog() -> Result<RuleCatalog> {
    let mut severity_by_rule = supported_rule_defaults();
    let supported_ids = severity_by_rule.keys().cloned().collect::<BTreeSet<_>>();
    let seeds = rule_seeds();
    let seed_ids = seeds
        .iter()
        .map(|seed| seed.id.to_string())
        .collect::<BTreeSet<_>>();

    if supported_ids != seed_ids {
        let missing = supported_ids
            .difference(&seed_ids)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let extra = seed_ids
            .difference(&supported_ids)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(miette!(
            "Rule catalog is out of sync. Missing metadata: [{}]. Extra metadata: [{}].",
            missing,
            extra
        ));
    }

    let known_groups = GROUPS
        .iter()
        .map(|group| group.key)
        .collect::<BTreeSet<_>>();
    let mut entries = Vec::with_capacity(seeds.len());

    for seed in seeds {
        if !known_groups.contains(seed.group_key) {
            return Err(miette!(
                "Unknown rule group `{}` in catalog",
                seed.group_key
            ));
        }
        let severity = severity_by_rule
            .remove(seed.id)
            .ok_or_else(|| miette!("Missing default severity for rule `{}`", seed.id))?;
        entries.push(RuleCatalogEntry {
            id: seed.id.to_string(),
            group_key: seed.group_key,
            scope: seed.scope,
            default_severity: severity,
            summary: summary_for_rule(seed.id),
            why: why_for_rule(seed.id),
        });
    }

    Ok(RuleCatalog {
        groups: GROUPS.to_vec(),
        entries,
    })
}

fn supported_rule_defaults() -> HashMap<String, Severity> {
    let mut defaults = HashMap::new();
    for rule in rules::all_builtin_rules() {
        defaults.insert(rule.name().to_string(), rule.default_severity());
    }
    for (rule_name, severity) in DEAD_CODE_RULES {
        defaults.insert(rule_name.to_string(), severity);
    }
    defaults
}

fn rule_seeds() -> Vec<RuleSeed> {
    let mut seeds = Vec::new();

    extend_group(
        &mut seeds,
        "core",
        RuleScope::Universal,
        &[
            "no-explicit-any",
            "no-console",
            "no-empty-catch",
            "prefer-const",
            "no-unused-vars",
            "unreachable-code",
            "no-missing-return",
        ],
    );
    extend_group(
        &mut seeds,
        "dead-code",
        RuleScope::Universal,
        &["unused-export", "unused-file", "unused-dependency"],
    );
    extend_group(
        &mut seeds,
        "universal-security",
        RuleScope::Universal,
        &["no-eval", "no-hardcoded-secrets"],
    );
    extend_group(
        &mut seeds,
        "js-performance",
        RuleScope::Universal,
        &[
            "no-chained-array-iterations",
            "prefer-tosorted",
            "no-regexp-in-loop",
            "prefer-math-min-max",
            "no-array-includes-in-loop",
            "no-sequential-style-assignment",
            "no-array-find-in-loop",
            "no-duplicate-storage-reads",
            "no-deep-nesting",
            "prefer-promise-all",
        ],
    );
    extend_group(
        &mut seeds,
        "react-hooks",
        RuleScope::React,
        &[
            "react/no-derived-state-effect",
            "react/no-fetch-in-effect",
            "react/no-cascading-set-state",
            "react/no-effect-event-handler",
            "react/no-derived-use-state",
            "react/prefer-use-reducer",
            "react/lazy-state-init",
            "react/functional-set-state",
            "react/unstable-deps",
        ],
    );
    extend_group(
        &mut seeds,
        "react-correctness",
        RuleScope::React,
        &[
            "react/no-array-index-key",
            "react/no-prevent-default",
            "react/no-conditional-render-zero",
        ],
    );
    extend_group(
        &mut seeds,
        "react-architecture",
        RuleScope::React,
        &[
            "react/no-giant-component",
            "react/no-render-in-render",
            "react/no-nested-component",
        ],
    );
    extend_group(
        &mut seeds,
        "react-performance",
        RuleScope::React,
        &[
            "react/no-usememo-simple-expr",
            "react/no-unstable-motion-props",
            "react/no-layout-animation",
            "react/no-animate-presence-in-list",
            "react/no-motion-in-list",
            "react/no-prop-on-memo",
            "react/no-hydration-flicker",
            "react/no-transition-all",
            "react/no-will-change",
            "react/no-blur-filter",
            "react/no-heavy-shadow",
            "react/no-barrel-import",
            "react/no-full-lodash",
            "react/no-moment",
            "react/prefer-dynamic-import",
            "react/use-lazy-motion",
            "react/no-undeferred-script",
        ],
    );
    extend_group(
        &mut seeds,
        "nextjs",
        RuleScope::NextJs,
        &[
            "nextjs/no-img-element",
            "nextjs/prefer-next-link",
            "nextjs/no-head-element",
            "nextjs/no-head-import",
            "nextjs/no-document-import",
            "nextjs/no-script-in-head",
            "nextjs/no-search-params-without-suspense",
            "nextjs/missing-metadata",
            "nextjs/no-side-effect-in-get-handler",
            "nextjs/no-async-client-component",
        ],
    );
    extend_group(
        &mut seeds,
        "server-components",
        RuleScope::NextJs,
        &[
            "react/server-auth-actions",
            "react/server-after-nonblocking",
        ],
    );
    extend_group(
        &mut seeds,
        "react-native",
        RuleScope::ReactNativeOrExpo,
        &[
            "react-native/no-inline-styles",
            "react-native/no-inline-callbacks",
            "react-native/no-anonymous-list-render",
            "react-native/no-scrollview-list",
            "react-native/no-raw-text",
            "react-native/no-alert",
            "react-native/no-image-uri-literal",
            "react-native/require-key-extractor",
        ],
    );
    extend_group(
        &mut seeds,
        "server-security",
        RuleScope::ServerFramework,
        &[
            "server/no-sql-injection",
            "server/no-shell-injection",
            "server/no-path-traversal",
            "server/no-unsafe-redirect",
            "server/no-cors-wildcard",
            "server/no-hardcoded-jwt-secret",
            "server/no-jwt-none-algorithm",
        ],
    );
    extend_group(
        &mut seeds,
        "server-reliability",
        RuleScope::ServerFramework,
        &[
            "server/no-unhandled-async-route",
            "server/no-swallowed-error",
            "server/no-process-exit-in-handler",
            "server/require-error-status",
            "server/no-throw-string",
        ],
    );
    extend_group(
        &mut seeds,
        "server-performance",
        RuleScope::ServerFramework,
        &[
            "server/no-n-plus-one",
            "server/no-unbounded-query",
            "server/no-sync-fs-in-handler",
            "server/no-blocking-crypto",
            "server/no-large-json-parse-sync",
        ],
    );
    extend_group(
        &mut seeds,
        "server-architecture",
        RuleScope::ServerFramework,
        &[
            "server/require-input-validation",
            "server/no-floating-transaction",
            "server/no-business-logic-in-route",
        ],
    );

    seeds
}

fn extend_group(
    seeds: &mut Vec<RuleSeed>,
    group_key: &'static str,
    scope: RuleScope,
    ids: &[&'static str],
) {
    seeds.extend(ids.iter().copied().map(|id| RuleSeed {
        id,
        group_key,
        scope,
    }));
}

fn summary_for_rule(rule_id: &str) -> String {
    let summary = match rule_id {
        "no-explicit-any" => "Avoid TypeScript `any` in favor of safer types.",
        "no-console" => "Flag console calls that leak debug output into runtime code.",
        "no-empty-catch" => "Catch blocks should handle, log, or rethrow errors.",
        "prefer-const" => "Use `const` when a binding is never reassigned.",
        "no-unused-vars" => "Surface declared bindings that are never used.",
        "unreachable-code" => "Flag statements that can never execute after control flow exits.",
        "no-missing-return" => {
            "Require consistent returns from functions that should produce a value."
        }
        "unused-export" => "Show exports that are never imported anywhere else in the project.",
        "unused-file" => "Show files that are never imported by any other file.",
        "unused-dependency" => "Show package.json dependencies that are never imported.",
        "no-eval" => "Block dynamic code execution APIs such as `eval` and string timers.",
        "no-hardcoded-secrets" => "Catch literal secrets committed directly in source.",
        "no-chained-array-iterations" => {
            "Prefer collapsing chained array passes into a single traversal."
        }
        "prefer-tosorted" => "Prefer `toSorted()` over clone-then-sort patterns.",
        "no-regexp-in-loop" => "Move regular expression construction out of hot loops.",
        "prefer-math-min-max" => {
            "Use `Math.min` or `Math.max` instead of sorting to pick extremes."
        }
        "no-array-includes-in-loop" => "Avoid repeated `includes()` lookups inside loops.",
        "no-sequential-style-assignment" => {
            "Batch style writes instead of mutating many properties one by one."
        }
        "no-array-find-in-loop" => {
            "Avoid `find()` scans inside loops when indexing or caching is cheaper."
        }
        "no-duplicate-storage-reads" => {
            "Reuse repeated storage reads instead of fetching the same key multiple times."
        }
        "no-deep-nesting" => "Flag deeply nested control flow that is hard to scan and optimize.",
        "prefer-promise-all" => "Run independent async work concurrently with `Promise.all`.",
        "react/no-derived-state-effect" => {
            "Derive render state directly instead of syncing it through effects."
        }
        "react/no-fetch-in-effect" => {
            "Move fetches out of `useEffect` when a better data flow exists."
        }
        "react/no-cascading-set-state" => {
            "Avoid stacking many state updates inside a single effect."
        }
        "react/no-effect-event-handler" => "Do not use `useEffect` as an event-handler wrapper.",
        "react/no-derived-use-state" => {
            "Avoid initializing state directly from props when it can be derived."
        }
        "react/prefer-use-reducer" => {
            "Prefer `useReducer` once state transitions become coordinated or complex."
        }
        "react/lazy-state-init" => "Use lazy initializers for expensive `useState` defaults.",
        "react/functional-set-state" => {
            "Use functional updates when the next state depends on the previous value."
        }
        "react/unstable-deps" => "Keep hook dependency arrays stable and explicit.",
        "react/no-array-index-key" => "Avoid array indexes as React keys on dynamic lists.",
        "react/no-prevent-default" => {
            "Be deliberate about canceling native browser behavior in JSX handlers."
        }
        "react/no-conditional-render-zero" => {
            "Avoid `.length && <JSX>` patterns that can render `0`."
        }
        "react/no-giant-component" => "Surface oversized components that need decomposition.",
        "react/no-render-in-render" => "Avoid calling `render*()` helpers inline from JSX trees.",
        "react/no-nested-component" => {
            "Keep component definitions at module scope, not inside renders."
        }
        "react/no-usememo-simple-expr" => {
            "Skip `useMemo` around trivial expressions that do not need caching."
        }
        "react/no-unstable-motion-props" => "Keep motion prop objects stable across renders.",
        "react/no-layout-animation" => "Avoid layout animations on hot React surfaces.",
        "react/no-animate-presence-in-list" => {
            "Do not wrap list rendering directly in `AnimatePresence`."
        }
        "react/no-motion-in-list" => "Keep motion-heavy elements out of large list renders.",
        "react/no-prop-on-memo" => "Avoid recreating props that defeat memoized components.",
        "react/no-hydration-flicker" => {
            "Flag patterns that cause visible client/server hydration flicker."
        }
        "react/no-transition-all" => "Avoid broad `transition: all` rules on interactive UI.",
        "react/no-will-change" => "Use `will-change` sparingly on React surfaces.",
        "react/no-blur-filter" => "Flag heavy blur filters that are expensive to render.",
        "react/no-heavy-shadow" => "Flag expensive shadow styles that hurt paint performance.",
        "react/no-barrel-import" => "Avoid barrel imports on hot paths when they inflate bundles.",
        "react/no-full-lodash" => "Avoid importing the full Lodash bundle.",
        "react/no-moment" => "Avoid Moment.js where lighter date utilities are enough.",
        "react/prefer-dynamic-import" => "Load heavy dependencies lazily with dynamic imports.",
        "react/use-lazy-motion" => "Use `LazyMotion` when shipping Framer Motion to the client.",
        "react/no-undeferred-script" => {
            "Defer non-critical scripts so they do not block rendering."
        }
        "nextjs/no-img-element" => "Prefer `next/image` over raw `<img>` in Next.js apps.",
        "nextjs/prefer-next-link" => "Use `next/link` for internal navigation.",
        "nextjs/no-head-element" => "Avoid raw `<head>` usage in Next.js components.",
        "nextjs/no-head-import" => "Avoid `next/head` in modern App Router code.",
        "nextjs/no-document-import" => "Restrict `next/document` imports to custom document files.",
        "nextjs/no-script-in-head" => "Avoid placing `<script>` tags directly inside `<head>`.",
        "nextjs/no-search-params-without-suspense" => {
            "Wrap `useSearchParams()` consumers in `Suspense`."
        }
        "nextjs/missing-metadata" => "Require route metadata on pages that should declare it.",
        "nextjs/no-side-effect-in-get-handler" => {
            "Keep GET handlers free of mutations and side effects."
        }
        "nextjs/no-async-client-component" => "Client components should not be declared `async`.",
        "react/server-auth-actions" => "Require auth checks before mutating inside server actions.",
        "react/server-after-nonblocking" => {
            "Wrap non-blocking side effects in `after()` inside server actions."
        }
        "react-native/no-inline-styles" => "Avoid inline style objects on React Native surfaces.",
        "react-native/no-inline-callbacks" => {
            "Avoid recreating inline callbacks on native components."
        }
        "react-native/no-anonymous-list-render" => {
            "Keep `renderItem` stable instead of recreating it inline."
        }
        "react-native/no-scrollview-list" => {
            "Prefer virtualized lists over mapping large collections in `ScrollView`."
        }
        "react-native/no-raw-text" => {
            "Wrap text in `Text` instead of rendering raw strings directly."
        }
        "react-native/no-alert" => "Avoid platform alerts as a default UX escape hatch.",
        "react-native/no-image-uri-literal" => "Avoid hardcoded remote image URIs in JSX.",
        "react-native/require-key-extractor" => {
            "Provide `keyExtractor` for virtualized React Native lists."
        }
        "server/no-sql-injection" => "Avoid building SQL with request-controlled interpolation.",
        "server/no-shell-injection" => "Do not pass dynamic input to shell execution APIs.",
        "server/no-path-traversal" => "Avoid feeding request data directly into filesystem paths.",
        "server/no-unsafe-redirect" => "Do not redirect to request-controlled destinations.",
        "server/no-cors-wildcard" => "Do not combine wildcard CORS origins with credentials.",
        "server/no-hardcoded-jwt-secret" => "Avoid shipping hardcoded JWT signing secrets.",
        "server/no-jwt-none-algorithm" => "Reject insecure JWT `none` algorithm usage.",
        "server/no-unhandled-async-route" => {
            "Async route handlers should catch and translate failures."
        }
        "server/no-swallowed-error" => {
            "Catch blocks must surface, handle, or rethrow server errors."
        }
        "server/no-process-exit-in-handler" => {
            "Never terminate the process from inside a request handler."
        }
        "server/require-error-status" => {
            "Set an HTTP error status before returning error payloads."
        }
        "server/no-throw-string" => "Throw `Error` objects instead of raw strings.",
        "server/no-n-plus-one" => "Avoid database or network requests inside loops.",
        "server/no-unbounded-query" => "Bound large collection queries with limits or pagination.",
        "server/no-sync-fs-in-handler" => "Avoid sync filesystem work in request handlers.",
        "server/no-blocking-crypto" => "Avoid blocking crypto work in server request paths.",
        "server/no-large-json-parse-sync" => {
            "Avoid large synchronous JSON parsing in hot handlers."
        }
        "server/require-input-validation" => {
            "Validate request bodies before using them in handlers."
        }
        "server/no-floating-transaction" => {
            "Wrap coordinated ORM mutations in an explicit transaction."
        }
        "server/no-business-logic-in-route" => {
            "Push oversized route logic into services or helpers."
        }
        _ => fallback_summary(rule_id),
    };

    summary.to_string()
}

pub(crate) fn why_for_rule(rule_id: &str) -> String {
    let why = match rule_id {
        // ── Core ──
        "no-explicit-any" => {
            "`any` disables type checking wherever it spreads, hiding bugs the compiler would otherwise catch for free."
        }
        "no-console" => {
            "Leftover console calls clutter production logs and can accidentally leak sensitive data to the browser."
        }
        "no-empty-catch" => {
            "Silently swallowing errors makes failures invisible, turning small issues into hard-to-trace outages."
        }
        "prefer-const" => {
            "Signals intent clearly — readers instantly know a `const` binding won't change, reducing cognitive load."
        }
        "no-unused-vars" => {
            "Dead bindings add noise and suggest incomplete refactors, making the code harder to trust and navigate."
        }
        "unreachable-code" => {
            "Code after a return or throw never runs; its presence misleads readers and hides logic errors."
        }
        "no-missing-return" => {
            "A function that sometimes returns a value and sometimes doesn't produces subtle `undefined` bugs at call sites."
        }

        // ── Dead Code ──
        "unused-export" => {
            "Exports nobody imports are dead weight — they bloat bundles and make the public API look bigger than it is."
        }
        "unused-file" => {
            "Orphaned files slow down tooling, confuse new contributors, and accumulate maintenance cost for zero value."
        }
        "unused-dependency" => {
            "Phantom dependencies inflate install size and attack surface while providing nothing in return."
        }

        // ── Universal Security ──
        "no-eval" => {
            "Dynamic code execution is the most direct path to code-injection attacks and defeats all static analysis."
        }
        "no-hardcoded-secrets" => {
            "Secrets in source get committed to git history forever, making credential rotation painful and leaks likely."
        }

        // ── JS Performance ──
        "no-chained-array-iterations" => {
            "Each chained pass allocates a new array and walks every element again — collapsing them cuts both memory and CPU."
        }
        "prefer-tosorted" => {
            "`toSorted()` avoids a manual clone step and communicates immutable intent, reducing allocation mistakes."
        }
        "no-regexp-in-loop" => {
            "Constructing a regex on every iteration recompiles the pattern repeatedly — hoisting it is a free speedup."
        }
        "prefer-math-min-max" => {
            "Sorting an array just to pick the smallest or largest value is O(n log n) when O(n) will do."
        }
        "no-array-includes-in-loop" => {
            "Calling `includes()` inside a loop turns an O(n) scan into O(n\u{00b2}) — a Set lookup keeps it O(n)."
        }
        "no-sequential-style-assignment" => {
            "Multiple individual style writes trigger layout recalculations each time; batching them lets the browser optimize."
        }
        "no-array-find-in-loop" => {
            "Repeated `find()` inside a loop re-scans the array on every pass — pre-index it once instead."
        }
        "no-duplicate-storage-reads" => {
            "Each storage read is a synchronous I/O call; caching the result avoids redundant blocking work."
        }
        "no-deep-nesting" => {
            "Deeply nested blocks are hard to reason about and tend to hide bugs — flattening improves readability and testability."
        }
        "prefer-promise-all" => {
            "Sequential awaits waste time waiting for independent work — `Promise.all` runs them in parallel."
        }

        // ── React Hooks ──
        "react/no-derived-state-effect" => {
            "Effects that sync derived state cause an extra render cycle; computing it inline is simpler and faster."
        }
        "react/no-fetch-in-effect" => {
            "Fetches in effects lack cancellation and race-condition handling that dedicated data libraries provide out of the box."
        }
        "react/no-cascading-set-state" => {
            "Stacking multiple state updates inside one effect triggers cascading re-renders that degrade performance."
        }
        "react/no-effect-event-handler" => {
            "Using an effect as a proxy for an event handler adds unnecessary complexity and an extra render cycle."
        }
        "react/no-derived-use-state" => {
            "State initialized from props goes stale when props change — deriving it during render keeps it in sync automatically."
        }
        "react/prefer-use-reducer" => {
            "Multiple interrelated `useState` calls make state transitions hard to follow; a reducer centralizes the logic."
        }
        "react/lazy-state-init" => {
            "Expensive initializers run on every render unless wrapped in a function — lazy init runs them only once."
        }
        "react/functional-set-state" => {
            "Closures capture stale state; functional updates always read the latest value, preventing subtle data-loss bugs."
        }
        "react/unstable-deps" => {
            "Unstable dependencies cause hooks to re-fire every render, wasting work and risking infinite loops."
        }

        // ── React Correctness ──
        "react/no-array-index-key" => {
            "Index keys break identity on reorder or deletion, causing React to mismatch state with the wrong element."
        }
        "react/no-prevent-default" => {
            "Blanket `preventDefault` on native elements breaks accessibility — forms won't submit, links won't navigate."
        }
        "react/no-conditional-render-zero" => {
            "`.length && <JSX>` renders the literal `0` when the array is empty, producing a visible zero in the UI."
        }

        // ── React Architecture ──
        "react/no-giant-component" => {
            "Oversized components are hard to test, reuse, and review — splitting them sharpens responsibility boundaries."
        }
        "react/no-render-in-render" => {
            "Inline render helpers re-create JSX trees every render, defeating reconciliation and losing local state."
        }
        "react/no-nested-component" => {
            "Components defined inside renders unmount and remount every cycle, destroying state and hurting performance."
        }

        // ── React Performance ──
        "react/no-usememo-simple-expr" => {
            "Memoizing a trivial expression costs more than recomputing it — the hook overhead outweighs any savings."
        }
        "react/no-unstable-motion-props" => {
            "Recreating motion prop objects every render restarts animations and defeats Framer Motion's diffing."
        }
        "react/no-layout-animation" => {
            "Layout animations force the browser to recalculate geometry on every frame, causing jank on busy pages."
        }
        "react/no-animate-presence-in-list" => {
            "AnimatePresence around an entire list re-diffs every child on any change, turning a list update into an O(n) animation pass."
        }
        "react/no-motion-in-list" => {
            "Motion wrappers on every list item multiply animation bookkeeping — use them selectively on key elements."
        }
        "react/no-prop-on-memo" => {
            "Passing a new object or function literal to a memoized component invalidates the memo on every render."
        }
        "react/no-hydration-flicker" => {
            "Client-only values that differ from the server HTML cause a visible flash when React hydrates the page."
        }
        "react/no-transition-all" => {
            "`transition: all` forces the browser to animate every property change, including layout-triggering ones."
        }
        "react/no-will-change" => {
            "Overusing `will-change` promotes elements to GPU layers, consuming memory and sometimes hurting instead of helping."
        }
        "react/no-blur-filter" => {
            "Blur filters are among the most expensive CSS operations — they repaint large pixel areas every frame."
        }
        "react/no-heavy-shadow" => {
            "Large or layered shadows increase paint time significantly, especially on low-end devices."
        }
        "react/no-barrel-import" => {
            "Barrel files pull in every re-export, preventing tree-shaking and inflating the bundle with unused code."
        }
        "react/no-full-lodash" => {
            "Importing top-level Lodash bundles the entire library (~70 KB) when you likely need a single function."
        }
        "react/no-moment" => {
            "Moment.js is 300 KB+ and mutable by design — modern alternatives are smaller, faster, and immutable."
        }
        "react/prefer-dynamic-import" => {
            "Statically importing heavy libraries blocks the initial bundle — lazy loading defers the cost until needed."
        }
        "react/use-lazy-motion" => {
            "LazyMotion loads only the Framer Motion features you use, cutting its bundle contribution significantly."
        }
        "react/no-undeferred-script" => {
            "Non-deferred scripts block HTML parsing, delaying first paint and time-to-interactive."
        }

        // ── Next.js ──
        "nextjs/no-img-element" => {
            "`next/image` auto-optimizes format, size, and lazy loading — raw `<img>` skips all of that."
        }
        "nextjs/prefer-next-link" => {
            "`next/link` enables client-side transitions and prefetching; plain `<a>` triggers full page reloads."
        }
        "nextjs/no-head-element" => {
            "Raw `<head>` conflicts with Next.js's built-in head management and can cause duplicate or missing meta tags."
        }
        "nextjs/no-head-import" => {
            "`next/head` is a Pages Router API — App Router uses metadata exports, and mixing them causes conflicts."
        }
        "nextjs/no-document-import" => {
            "`next/document` only works in `_document` — importing it elsewhere silently fails or breaks SSR."
        }
        "nextjs/no-script-in-head" => {
            "Scripts in `<head>` block rendering; use `next/script` with a loading strategy instead."
        }
        "nextjs/no-search-params-without-suspense" => {
            "Without Suspense, `useSearchParams` forces the entire page into client-side rendering, breaking static optimization."
        }
        "nextjs/missing-metadata" => {
            "Pages without metadata hurt SEO and social sharing — Next.js metadata API makes it declarative and easy."
        }
        "nextjs/no-side-effect-in-get-handler" => {
            "GET handlers can be cached and replayed — mutations inside them may execute unpredictably or not at all."
        }
        "nextjs/no-async-client-component" => {
            "Client components cannot be async — React will throw at runtime, but the error is easy to miss during development."
        }

        // ── Server Components ──
        "react/server-auth-actions" => {
            "Server actions run with full backend privileges — skipping auth checks means any caller can mutate your data."
        }
        "react/server-after-nonblocking" => {
            "`after()` defers side effects until after the response, keeping the user-facing response fast."
        }

        // ── React Native ──
        "react-native/no-inline-styles" => {
            "Inline style objects are re-created every render, defeating the native bridge's style caching."
        }
        "react-native/no-inline-callbacks" => {
            "New function references every render cause child components to re-render unnecessarily on native."
        }
        "react-native/no-anonymous-list-render" => {
            "An unstable `renderItem` remounts every cell on each render, destroying scroll position and local state."
        }
        "react-native/no-scrollview-list" => {
            "ScrollView renders all children at once — virtualized lists only render what's on screen, saving memory."
        }
        "react-native/no-raw-text" => {
            "Raw strings outside `<Text>` crash on native platforms — they must be wrapped to render correctly."
        }
        "react-native/no-alert" => {
            "Platform alerts block the JS thread and offer no customization — use a modal component instead."
        }
        "react-native/no-image-uri-literal" => {
            "Hardcoded URIs scatter asset management across files and make cache-busting and CDN changes painful."
        }
        "react-native/require-key-extractor" => {
            "Without `keyExtractor`, FlatList falls back to index keys which break identity on list mutations."
        }

        // ── Server Security ──
        "server/no-sql-injection" => {
            "Interpolating user input into SQL lets attackers read, modify, or delete your entire database."
        }
        "server/no-shell-injection" => {
            "Unsanitized input in shell commands lets attackers execute arbitrary code on your server."
        }
        "server/no-path-traversal" => {
            "Request-controlled paths like `../../etc/passwd` let attackers read or overwrite arbitrary files."
        }
        "server/no-unsafe-redirect" => {
            "Open redirects let attackers send users to phishing pages that appear to come from your domain."
        }
        "server/no-cors-wildcard" => {
            "Wildcard CORS with credentials lets any origin make authenticated requests to your API."
        }
        "server/no-hardcoded-jwt-secret" => {
            "A leaked signing secret lets anyone forge valid tokens — secrets must come from secure config."
        }
        "server/no-jwt-none-algorithm" => {
            "The `none` algorithm accepts unsigned tokens, letting anyone bypass authentication entirely."
        }

        // ── Server Reliability ──
        "server/no-unhandled-async-route" => {
            "Unhandled rejections in async handlers crash the process or return opaque 500s instead of useful errors."
        }
        "server/no-swallowed-error" => {
            "Swallowed errors make failures silent — problems compound until they surface somewhere harder to debug."
        }
        "server/no-process-exit-in-handler" => {
            "`process.exit` in a handler kills every in-flight request, not just the one that failed."
        }
        "server/require-error-status" => {
            "Returning an error body with a 200 status confuses clients and breaks error-handling middleware."
        }
        "server/no-throw-string" => {
            "Thrown strings lose the stack trace — `Error` objects preserve it, making debugging far easier."
        }

        // ── Server Performance ──
        "server/no-n-plus-one" => {
            "A query per loop iteration turns one request into hundreds of database round-trips, destroying latency."
        }
        "server/no-unbounded-query" => {
            "Loading an entire table into memory can exhaust RAM and block the event loop on large datasets."
        }
        "server/no-sync-fs-in-handler" => {
            "Sync filesystem calls block the event loop, stalling every other request until the I/O completes."
        }
        "server/no-blocking-crypto" => {
            "Blocking crypto (pbkdf2Sync, scryptSync) freezes the event loop for hundreds of milliseconds per call."
        }
        "server/no-large-json-parse-sync" => {
            "Parsing large JSON synchronously blocks the event loop, causing latency spikes for all concurrent requests."
        }

        // ── Server Architecture ──
        "server/require-input-validation" => {
            "Unvalidated input is the root cause of most security and data-integrity bugs — validate at the boundary."
        }
        "server/no-floating-transaction" => {
            "Without a transaction, a failure between two writes leaves your data in an inconsistent state."
        }
        "server/no-business-logic-in-route" => {
            "Logic buried in route handlers can't be reused or unit-tested — extracting it makes both possible."
        }

        _ => "This rule catches a pattern that commonly leads to bugs, performance issues, or maintenance headaches.",
    };

    why.to_string()
}

fn fallback_summary(rule_id: &str) -> &'static str {
    match rule_id {
        _ => "Inspect this rule in context to understand the behavior it enforces.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_all_supported_rules() {
        let catalog = build_rule_catalog().unwrap();
        let catalog_ids = catalog
            .entries
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<BTreeSet<_>>();
        let defaults = supported_rule_defaults();
        let supported = defaults
            .keys()
            .map(|id| id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(catalog_ids, supported);
    }

    #[test]
    fn representative_rules_map_to_expected_groups() {
        let catalog = build_rule_catalog().unwrap();
        let by_id = catalog
            .entries
            .iter()
            .map(|entry| (entry.id.as_str(), entry.group_key))
            .collect::<HashMap<_, _>>();
        assert_eq!(by_id.get("react/no-fetch-in-effect"), Some(&"react-hooks"));
        assert_eq!(
            by_id.get("server/no-sql-injection"),
            Some(&"server-security")
        );
        assert_eq!(by_id.get("nextjs/no-img-element"), Some(&"nextjs"));
    }

    #[test]
    fn representative_rules_use_engine_default_severity() {
        let catalog = build_rule_catalog().unwrap();
        let by_id = catalog
            .entries
            .iter()
            .map(|entry| (entry.id.as_str(), &entry.default_severity))
            .collect::<HashMap<_, _>>();
        assert_eq!(by_id.get("no-unused-vars"), Some(&&Severity::Error));
        assert_eq!(by_id.get("no-console"), Some(&&Severity::Warning));
        assert_eq!(
            by_id.get("server/no-sql-injection"),
            Some(&&Severity::Error)
        );
    }

    #[test]
    fn scope_matches_project_detection() {
        assert!(RuleScope::Universal.applies_to_project(&ProjectInfo::default()));
        assert!(RuleScope::React.applies_to_project(&ProjectInfo::test_react()));
        assert!(!RuleScope::NextJs.applies_to_project(&ProjectInfo::test_react()));
        assert!(RuleScope::ServerFramework.applies_to_project(&ProjectInfo::test_server()));
    }
}
