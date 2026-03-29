# Zarc

Blazing-fast JavaScript and TypeScript linting with a deliberately narrow MVP surface.

## Current Product Direction

Zarc is focused on one job:

- lint JS/TS files fast
- keep startup overhead low
- keep the rule set small and high-value
- optimize cold CLI performance first

Code health scoring is intentionally out of scope. Zarc now also supports a narrow plain-English
rule surface that compiles into deterministic native checks.

## Commands

```bash
# Initialize config
zarc init

# Lint the current project
zarc check .

# Lint a specific directory
zarc check ./src

# Print JSON output
zarc check . --format json

# Skip the adaptive cache for a cold-run measurement
zarc check . --no-cache --timing
```

By default, `zarc check` uses an adaptive cache. It reuses cached results when that is predicted to beat a cold run and bypasses the cache when the cache overhead would likely lose.

## Configuration

Zarc uses `zarc.toml`:

```toml
[lint]
rules = { no-explicit-any = "warn", no-unused-vars = "error", no-console = "warn", prefer-const = "warn", no-empty-catch = "error" }

[[lint.english_rules]]
text = "no function should have more than 3 params"
severity = "warn"

[files]
exclude = ["node_modules", "dist", "build", ".git"]
```

If you want broader natural-language compilation for English rules, add a Zarc API key in
`.zarcrc`:

```toml
api_key = "zk_your_zarc_api_key"
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

Custom English rules compile once into a cached internal rule IR and then execute inside Zarc's
normal OXC-backed lint pass. The compiler keeps a native handwritten fast path for canonical
templates and can optionally fall back to Zarc's hosted compiler during compilation when a Zarc API
key is available in `.zarcrc`. Compiled IR is cached, so there is no per-file remote call in the
lint execution path.

Supported native predicate forms:

- `no function should have more than 3 params`
- `do not import lodash`
- `do not call console.log`
- `do not use console.log`
- `function names should start with use`
- `function names should end with service`
- `no file should have more than 400 lines`

Each English rule compiles to an ID like `english/<kind>/<hash>`; place that ID in `[lint].rules` to enable/disable the rule and choose `warn`/`error`. By default the rule uses the severity declared in the `[[lint.english_rules]]` block.

`.zarcrc` is resolved project-first and then from `~/.zarcrc`. When a Zarc API key is available,
semantically equivalent phrasing can compile into the same native predicates even if it does not
match the canonical templates exactly. Unsupported or ambiguous English rules still fail fast
during compilation instead of being approximated.

## File Support

Zarc currently discovers and lints:

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
zarc/
├── src/
│   ├── lib.rs            # CLI orchestration and output
│   ├── main.rs           # Binary entrypoint
│   ├── cli.rs            # CLI args, config loading, file discovery
│   └── rules/            # Built-in rules, English-rule compiler, and cache
└── zarc-npm/             # npm wrapper package for native binaries
```

## Performance Priorities

- one parse per file
- file-level parallelism
- minimal config surface
- adaptive cache
- no non-lint product overhead in the MVP

## Roadmap

- [x] Rename public product surface to `zarc`
- [x] Remove code health scoring from the MVP
- [x] Support JS and TS file discovery
- [x] Add a minimal `zarc.toml`
- [x] Add cached plain-English rule compilation for supported native checks
- [ ] Replace the current heuristic rule implementations with proper AST/semantic analysis
- [ ] Reduce the shipped rule set to the few rules we want to support extremely well
- [ ] Tighten ignore handling and config-driven rule enablement further

## License

MIT
