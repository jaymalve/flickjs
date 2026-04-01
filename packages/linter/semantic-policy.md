# Semantic Policy Refinement Example

This file shows one concrete example of how semantic policy refinement would be implemented in Flint.

## Example Rule

`no-unused-vars`

This is the best example because the rule is not just about syntax. It depends on:

- what kind of symbol something is
- how that symbol is referenced
- whether type-only usage counts
- whether some names should be intentionally ignored

## Problem

Suppose Flint currently flags this:

```ts
import type { Foo } from "./types";

type Bar = Foo;
```

and later you decide this policy:

- imported symbols used in type positions should count as used, regardless of `import` vs `import type`
- catch variables named `error` should be ignored by policy
- `_`-prefixed names should still be ignored

That is not a parser change. That is a semantic policy change.

## Step 1: Write The Policy

Before touching the rule code, define the behavior clearly.

### Desired behavior

- `import type { Foo } ...` and later `type X = Foo`:
  - no diagnostic
- `import { Foo } ...` and later `type X = Foo`:
  - no diagnostic
- `import { Foo } ...` and later `create<Foo>()`:
  - no diagnostic
- `catch (error) { ... }` with no read of `error`:
  - no diagnostic if Flint chooses to exempt `error`
- `function run(_unused) {}`:
  - no diagnostic

## Step 2: Add Tests First

You add tests that express the intended behavior before changing the implementation.

Example tests:

```rust
#[test]
fn keeps_type_only_imports_used_in_type_positions() {
    let messages = unused_var_messages(
        "test.ts",
        "import type { Foo } from './types';\ntype Bar = Foo;\n",
    );
    assert!(messages.is_empty());
}

#[test]
fn keeps_value_imports_used_only_in_type_positions() {
    let messages = unused_var_messages(
        "test.ts",
        "import { Foo } from './types';\ntype Bar = Foo;\n",
    );
    assert!(messages.is_empty());
}

#[test]
fn keeps_value_imports_used_in_generic_type_arguments() {
    let messages = unused_var_messages(
        "test.ts",
        "import { Foo } from './types';\nconst value = create<Foo>();\n",
    );
    assert!(messages.is_empty());
}

#[test]
fn ignores_error_catch_binding_by_policy() {
    let messages = unused_var_messages(
        "test.js",
        "try { work(); } catch (error) { recover(); }\n",
    );
    assert!(messages.is_empty());
}
```

## Step 3: Refine The Semantic Helpers

Then you update the rule logic in `src/rules/no_unused_vars.rs`.

Today the helper might be conceptually simple:

```rust
fn has_meaningful_usage(
    ctx: &LintContext,
    symbol_id: SymbolId,
    flags: SymbolFlags,
) -> bool {
    ctx.semantic
        .scoping()
        .get_resolved_references(symbol_id)
        .any(|reference| reference.is_read() || (flags.is_import() && reference.is_type()))
}
```

If the policy becomes more explicit, you refine it into something like:

```rust
fn has_meaningful_usage(
    ctx: &LintContext,
    symbol_id: SymbolId,
    flags: SymbolFlags,
) -> bool {
    ctx.semantic
        .scoping()
        .get_resolved_references(symbol_id)
        .any(|reference| {
            if flags.is_import() {
                reference.is_type() || reference.is_read()
            } else {
                reference.is_read() && reference.is_value()
            }
        })
}

fn should_ignore_unused_symbol(name: &str, flags: SymbolFlags) -> bool {
    if name.starts_with('_') {
        return true;
    }

    if flags.is_catch_variable() && name == "error" {
        return true;
    }

    false
}
```

And then in the main rule body:

```rust
let flags = scoping.symbol_flags(symbol_id);
let name = scoping.symbol_name(symbol_id);

if should_ignore_unused_symbol(name, flags) {
    return None;
}

if has_meaningful_usage(ctx, symbol_id, flags) {
    return None;
}
```

## Step 4: Verify Against Real Code

After the unit tests pass, run Flint on a few real projects and inspect:

- false positives
- false negatives
- cases where the policy feels too strict or too loose

If a real-world case appears repeatedly, decide whether:

- the policy should change
- or the project code should change

That distinction matters. A linter should not silently drift just because one repo had a weird pattern.

## What This Means

Semantic policy refinement in Flint is this loop:

1. Define exact behavior.
2. Add a failing test.
3. Refine semantic helper logic.
4. Run tests and real-project checks.
5. Keep only behavior that is both intentional and defensible.

## Why This Matters

This is how Flint becomes trustworthy.

The parser and AST walker can already find symbols and references. The real product quality comes from deciding what those references mean for each rule, then locking that behavior down with tests.

---

## Second Example Rule

`prefer-const`

This rule is another good example because it looks simple at first, but correct behavior depends on semantic meaning, not just syntax.

## Problem

Suppose Flint currently flags simple cases like:

```js
let count = 1;
console.log(count);
```

but later you want to refine the policy for more complex bindings:

- destructuring should be flagged only if every bound name is never reassigned
- `for` loop bindings should stay exempt
- bindings changed inside closures should still count as reassigned
- destructuring with rest elements should be skipped in v1 if the behavior feels too risky

Again, this is not a parser problem. It is a semantic policy decision.

## Step 1: Write The Policy

Define what Flint means by "prefer const".

### Desired behavior

- `let count = 1; console.log(count);`
  - diagnostic
- `let count = 1; count += 1;`
  - no diagnostic
- `let { a, b } = obj; use(a, b);`
  - diagnostic if neither `a` nor `b` is reassigned
- `let { a, ...rest } = obj;`
  - skip in v1 if rest-pattern behavior is intentionally unsupported
- `let value = 1; setTimeout(() => value = 2);`
  - no diagnostic, because the symbol is mutated later
- `for (let i = 0; i < 10; i++) {}`
  - no diagnostic

## Step 2: Add Tests First

Before changing the rule code, encode the target behavior.

Example tests:

```rust
#[test]
fn flags_destructured_bindings_when_none_are_reassigned() {
    let messages = prefer_const_messages(
        "let { a, b } = obj;\nuse(a, b);\n",
    );
    assert_eq!(messages.len(), 1);
}

#[test]
fn ignores_destructuring_when_any_binding_is_reassigned() {
    let messages = prefer_const_messages(
        "let { a, b } = obj;\na = 1;\nuse(b);\n",
    );
    assert!(messages.is_empty());
}

#[test]
fn ignores_loop_bindings() {
    let messages = prefer_const_messages(
        "for (let i = 0; i < 3; i++) { console.log(i); }\n",
    );
    assert!(messages.is_empty());
}

#[test]
fn ignores_bindings_mutated_inside_closures() {
    let messages = prefer_const_messages(
        "let value = 1;\nqueueMicrotask(() => { value = 2; });\n",
    );
    assert!(messages.is_empty());
}
```

## Step 3: Refine The Semantic Helpers

The existing implementation in `src/rules/prefer_const.rs` already uses symbol mutation tracking. That is the correct foundation.

The basic logic looks like this:

```rust
let bindings = declarator.id.get_binding_identifiers();

if bindings.iter().any(|binding| {
    binding
        .symbol_id
        .get()
        .is_none_or(|symbol_id| ctx.semantic.scoping().symbol_is_mutated(symbol_id))
}) {
    return None;
}
```

If you later decide to skip object or array patterns with rest elements in v1, you would add a policy helper before this mutation check.

Example shape:

```rust
fn should_skip_pattern(pattern: &BindingPattern) -> bool {
    match &pattern.kind {
        BindingPatternKind::ObjectPattern(object) => object.rest.is_some(),
        BindingPatternKind::ArrayPattern(array) => array.rest.is_some(),
        _ => false,
    }
}
```

Then gate the rule:

```rust
if should_skip_pattern(&declarator.id) {
    return None;
}
```

If later you decide partial destructuring should still be flagged when only some bindings are mutable, you would change the policy explicitly.

Strict policy:

- only flag when all bound names are stable

Alternative policy:

- flag only the stable sub-bindings and attach diagnostics to each name

Those are different product decisions. The semantic layer enables both, but Flint has to choose one.

## Step 4: Validate On Real Code

Run Flint on real repos and look specifically for:

- false positives on loop variables
- false positives on destructuring
- missed mutations through nested functions
- assignments via update expressions like `count++`
- compound writes like `count += 1`

If those behave correctly, the rule is close to trustworthy. If they do not, the fix should happen in the semantic policy helpers, not through string heuristics.

## What This Means

For `prefer-const`, semantic policy refinement is about deciding:

- which declaration forms are safe to analyze
- which mutation patterns count
- which contexts should be exempt

Then you encode those choices directly around symbol mutation tracking and binding-pattern handling.

That is the same overall loop:

1. Define the exact behavior.
2. Add tests.
3. Refine semantic checks.
4. Validate against real code.
