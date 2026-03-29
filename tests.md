# Tests Added

This file lists the focused rule tests that were added, and why each one exists.

## `no-explicit-any`

Defined in [src/rules/no_explicit_any.rs](/Users/jaymalave/Desktop/ZarcDev/cova/src/rules/no_explicit_any.rs).

- `flags_any_in_multiple_type_positions`
Why: proves we are catching real `TSAnyKeyword` nodes across aliases, params, returns, and assertions.

- `ignores_unknown_and_plain_identifiers`
Why: proves we do not confuse identifier names like `any` with the actual TypeScript `any` type.

## `no-console`

Defined in [src/rules/no_console.rs](/Users/jaymalave/Desktop/ZarcDev/cova/src/rules/no_console.rs).

- `flags_console_calls`
Why: proves global `console.*(...)` calls are detected.

- `ignores_shadowed_console`
Why: proves semantic resolution works and we do not flag a locally shadowed `console`.

## `no-empty-catch`

Defined in [src/rules/no_empty_catch.rs](/Users/jaymalave/Desktop/ZarcDev/cova/src/rules/no_empty_catch.rs).

- `flags_empty_catch_blocks`
Why: proves empty catch clauses are reported.

- `ignores_non_empty_catch_blocks`
Why: proves any real statement in the catch body suppresses the lint.

## `prefer-const`

Defined in [src/rules/prefer_const.rs](/Users/jaymalave/Desktop/ZarcDev/cova/src/rules/prefer_const.rs).

- `flags_simple_let_without_reassignment`
Why: baseline `prefer-const` behavior.

- `flags_destructured_let_without_reassignment`
Why: proves the rule now handles destructuring, not just simple identifiers.

- `ignores_reassigned_bindings`
Why: proves semantic mutation tracking is respected.

- `ignores_loop_bindings`
Why: proves the deliberate loop exemption is enforced.

## `no-unused-vars`

Defined in [src/rules/no_unused_vars.rs](/Users/jaymalave/Desktop/ZarcDev/cova/src/rules/no_unused_vars.rs).

- `flags_unused_destructured_binding`
Why: proves destructured names are tracked individually.

- `flags_unused_parameter`
Why: proves normal function params are covered.

- `ignores_underscore_prefixed_parameter`
Why: proves the `_name` ignore policy works.

- `flags_unused_catch_binding`
Why: proves catch bindings are included now.

- `keeps_type_only_imports_used_in_type_positions`
Why: proves TypeScript type-only import semantics are handled correctly.

## Test Helper

Defined in [src/rules/mod.rs](/Users/jaymalave/Desktop/ZarcDev/cova/src/rules/mod.rs).

- `lint_source_for_test`
Why: lets tests run the full parser + semantic + rule pipeline on source strings without needing fixture files on disk.

## Why These Tests Exist

These tests were added to pin down the exact semantic behavior we implemented, especially around:

- destructuring
- symbol shadowing
- catch bindings
- mutation tracking
- TypeScript type usage

They are intentionally narrow and rule-focused rather than broad integration tests.

## How To Run All Tests

From the repo root at [/Users/jaymalave/Desktop/ZarcDev/cova](/Users/jaymalave/Desktop/ZarcDev/cova), run:

```bash
cargo test
```

That runs all Rust tests in the project at once.

Useful variants:

```bash
cargo test -- --nocapture
cargo test no_unused_vars
cargo test prefer_const
cargo test --lib
```

If you want a quick correctness pass before running the full suite:

```bash
cargo check
cargo build
```
