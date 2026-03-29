# Zarc

Blazing-fast JavaScript and TypeScript linting with a deliberately narrow MVP surface.

## Current Product Direction

Zarc is focused on one job:

- lint JS/TS files fast
- keep startup overhead low
- keep the rule set small and high-value
- optimize cold CLI performance first

Code health scoring and natural-language rules are intentionally out of scope for this MVP.

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

# Skip the cache for a cold-run measurement
zarc check . --no-cache --timing
```

## Configuration

Zarc uses `zarc.toml`:

```toml
[lint]
rules = { no-explicit-any = "warn", no-unused-vars = "error", no-console = "warn", prefer-const = "warn", no-empty-catch = "error" }

[files]
exclude = ["node_modules", "dist", "build", ".git"]
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
│   └── rules/            # Built-in lint rules and cache
└── zarc-npm/             # npm wrapper package for native binaries
```

## Performance Priorities

- one parse per file
- file-level parallelism
- minimal config surface
- optional cache
- no non-lint product overhead in the MVP

## Roadmap

- [x] Rename public product surface to `zarc`
- [x] Remove code health scoring from the MVP
- [x] Remove natural-language rule compilation from the MVP
- [x] Support JS and TS file discovery
- [x] Add a minimal `zarc.toml`
- [ ] Replace the current heuristic rule implementations with proper AST/semantic analysis
- [ ] Reduce the shipped rule set to the few rules we want to support extremely well
- [ ] Tighten ignore handling and config-driven rule enablement further

## License

MIT
