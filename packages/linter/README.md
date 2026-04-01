# Flint

Blazing-fast JavaScript and TypeScript linting with a deliberately narrow MVP surface.

## Current Product Direction

Flint is focused on one job:

- lint JS/TS files fast
- keep startup overhead low
- keep the rule set small and high-value
- optimize cold CLI performance first

Code health scoring is intentionally out of scope. Flint now also supports plain-English policies
that compile into deterministic native rule IR.

## Commands

```bash
# Initialize config
flint init

# Lint the current project
flint check .

# Lint a specific directory
flint check ./src

# Print JSON output
flint check . --format json

# Skip the adaptive cache for a cold-run measurement
flint check . --no-cache --timing
```

By default, `flint check` uses an adaptive cache. It reuses cached results when that is predicted to beat a cold run and bypasses the cache when the cache overhead would likely lose.

## Configuration

Flint uses `flint.toml`:

```toml
[lint]
rules = { no-explicit-any = "warn", no-unused-vars = "error", no-console = "warn", prefer-const = "warn", no-empty-catch = "error" }

[[lint.english_rules]]
text = "no function should have more than 3 params"
severity = "warn"

[files]
exclude = ["node_modules", "dist", "build", ".git"]
```

If you want broader natural-language compilation for English rules, add a Flint API key in
`.flintrc`:

```toml
api_key = "zk_your_flint_api_key"
```

Severity values:

- `off`
- `warn`
- `error`

## Built-in Rules

- `no-explicit-any`
- `no-console`
- `no-empty-catch`
- `prefer-const`
- `no-unused-vars`

All built-in rules are active. `prefer-const` and `no-unused-vars` currently use lightweight MVP heuristics and should later be replaced with full semantic implementations.

## Plain-English Rules

Custom English rules now compile into a typed native policy IR and then execute inside Flint's
normal OXC-backed lint pass. The hosted compiler is compile-time only: lint execution remains fully
local, deterministic, and millisecond-scale. Compiled policy artifacts are cached, so there is no
per-file remote call in the lint execution path.

The first-wave policy IR supports these native categories:

- AST and syntax rules
- import and module rules
- naming rules
- file and path rules
- comment and text rules
- file-local semantic rules

Canonical handwritten fast-path examples:

- `no function should have more than 3 params`
- `do not import lodash`
- `do not call console.log`
- `do not use console.log`
- `function names should start with use`
- `function names should end with service`
- `no file should have more than 400 lines`
- `no comments in files`
- `do not use todo in comments`

Each English rule compiles to an ID like `policy/<category>/<kind>/<hash>`; place that ID in
`[lint].rules` to enable/disable the rule and choose `warn`/`error`. By default the rule uses the
severity declared in the `[[lint.english_rules]]` block.

`.flintrc` is resolved project-first and then from `~/.flintrc`. When a Flint API key is available,
semantically equivalent phrasing can compile into the same native policy IR even if it does not
match the canonical templates exactly. Unsupported or ambiguous English rules still fail fast
during compilation instead of being approximated.

This is a clean break from the old predicate-only artifact format. Existing compiled English-rule
artifacts are treated as stale and will be regenerated under the new policy schema.

## File Support

Flint currently discovers and lints:

- `.js`
- `.jsx`
- `.mjs`
- `.cjs`
- `.ts`
- `.tsx`
- `.mts`
- `.cts`

## Architecture

```text
flint/
├── src/
│   ├── lib.rs            # CLI orchestration and output
│   ├── main.rs           # Binary entrypoint
│   ├── cli.rs            # CLI args, config loading, file discovery
│   └── rules/            # Built-in rules, policy IR compiler, and runtime evaluators
└── flint-npm/             # npm wrapper package for native binaries
```

## Performance Priorities

- one parse per file
- file-level parallelism
- minimal config surface
- adaptive cache
- no non-lint product overhead in the MVP

## Roadmap

- [x] Rename public product surface to `flint`
- [x] Remove code health scoring from the MVP
- [x] Support JS and TS file discovery
- [x] Add a minimal `flint.toml`
- [x] Add cached plain-English rule compilation for supported native checks
- [ ] Replace the current heuristic rule implementations with proper AST/semantic analysis
- [ ] Reduce the shipped rule set to the few rules we want to support extremely well
- [ ] Tighten ignore handling and config-driven rule enablement further

## License

MIT
