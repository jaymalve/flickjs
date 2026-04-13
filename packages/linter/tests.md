# Tests Added

This file lists the focused rule tests that were added, and why each one exists.

## `no-explicit-any`

Defined in [src/rules/no_explicit_any.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/no_explicit_any.rs).

- `flags_any_in_multiple_type_positions`
  Why: proves we are catching real `TSAnyKeyword` nodes across aliases, params, returns, and assertions.

- `ignores_unknown_and_plain_identifiers`
  Why: proves we do not confuse identifier names like `any` with the actual TypeScript `any` type.

## `no-console`

Defined in [src/rules/no_console.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/no_console.rs).

- `flags_console_calls`
  Why: proves global `console.*(...)` calls are detected.

- `ignores_shadowed_console`
  Why: proves semantic resolution works and we do not flag a locally shadowed `console`.

## `no-empty-catch`

Defined in [src/rules/no_empty_catch.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/no_empty_catch.rs).

- `flags_empty_catch_blocks`
  Why: proves empty catch clauses are reported.

- `ignores_non_empty_catch_blocks`
  Why: proves any real statement in the catch body suppresses the lint.

## `prefer-const`

Defined in [src/rules/prefer_const.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/prefer_const.rs).

- `flags_simple_let_without_reassignment`
  Why: baseline `prefer-const` behavior.

- `flags_destructured_let_without_reassignment`
  Why: proves the rule now handles destructuring, not just simple identifiers.

- `ignores_reassigned_bindings`
  Why: proves semantic mutation tracking is respected.

- `ignores_loop_bindings`
  Why: proves the deliberate loop exemption is enforced.

## `no-unused-vars`

Defined in [src/rules/no_unused_vars.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/no_unused_vars.rs).

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

- `keeps_normal_imports_used_in_generic_type_arguments`
  Why: proves imported symbols referenced in generic type arguments are treated as used.

- `keeps_normal_imports_used_in_type_positions`
  Why: proves normal imports used only in TypeScript type positions are not falsely flagged.

## Test Helper

Defined in [src/rules/mod.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/mod.rs).

- `lint_source_for_test`
  Why: lets tests run the full parser + semantic + rule pipeline on source strings without needing fixture files on disk.

- `lint_source_for_test_with_english_rules`
  Why: lets tests exercise built-in rules and compiled English rules in the same native pass.

## `cli`

Defined in [src/cli.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/cli.rs).

- `project_flickrc_overrides_home_flickrc`
  Why: proves `.flickrc` resolution prefers the project key over the home-directory fallback.

- `falls_back_to_home_flickrc_when_project_missing`
  Why: proves hosted compiler auth still works when only `~/.flickrc` is present.

- `rejects_empty_api_key_in_flickrc`
  Why: proves malformed hosted-auth config fails clearly instead of silently disabling compilation.

## `english_rules`

Defined in [src/rules/policy.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/policy.rs).

- `compiles_max_function_params_rule`
  Why: proves supported English config compiles into the expected native policy IR.

- `rejects_unsupported_english_rule`
  Why: proves unsupported plain-English rules still fail fast when no hosted compiler auth is available.

- `unsupported_english_rule_points_to_flickrc_auth_flow`
  Why: proves the unsupported-rule guidance points users at `.flickrc` rather than provider config.

- `compiled_artifact_round_trips`
  Why: proves the compiled English-rule artifact persists and reloads from cache correctly.

- `llm_compiles_unsupported_phrasing_into_supported_predicate`
  Why: proves the hosted compiler path can map broader natural-language phrasing into the same native policy IR.

- `llm_errors_fail_closed`
  Why: proves invalid compiler responses stop compilation instead of silently approximating behavior.

- `compiled_artifact_invalidates_when_compiler_fingerprint_changes`
  Why: proves cached English-rule artifacts are invalidated when the compiler mode or prompt/schema fingerprint changes.

- `max_function_params_rule_reports_diagnostics`
  Why: proves a compiled threshold rule executes inside the native lint pass and emits English-rule diagnostics.

- `banned_import_rule_reports_diagnostics`
  Why: proves compiled import bans run against real import declarations.

- `function_name_prefix_rule_reports_diagnostics`
  Why: proves naming rules compile and execute against function declarations.

## Why These Tests Exist

These tests were added to pin down the exact semantic behavior we implemented, especially around:

- destructuring
- symbol shadowing
- catch bindings
- mutation tracking
- TypeScript type usage

They are intentionally narrow and rule-focused rather than broad integration tests.

## How To Run All Tests

From the repo root at [/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter), run:

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
