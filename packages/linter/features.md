**Direction**

Flick Scan already has the right kernel for an agent-native linter: one parse + semantic pass, perf-conscious execution, JSON output, and a latent fix model in [src/rules/mod.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/mod.rs):31. What it does not have yet is a machine contract. Right now it mostly tells a human “here is a warning.” An agent needs “here is the problem, here are the valid edits, here is how safe they are, here is what to verify next.”

Hard truth: “AI coding agent native” does not mean “chatty” or “natural-language rules.” Your own doc is already pointing in the better direction at [semantic-policy.md](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/semantic-policy.md):179: explicit policy, tests, semantic intent, trustworthiness. Keep the core deterministic. Let the model choose among structured actions, not invent lint semantics on the fly.

**Highest-Value Features**

1. Replace `LintDiagnostic` with an agent-grade schema. The current shape in [src/rules/mod.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/rules/mod.rs):31 is too thin. Add stable issue ID, byte start/end, AST node kind, symbol/module refs, confidence, blast radius, root-cause group, and required verification steps.
2. Turn `fix: Option<Fix>` into `fixes: Vec<FixCandidate>`. Agents need ranked options, not one patch. Add safety classes like `exact_safe`, `semantic_safe`, `risky_refactor`, and `suppress_only`.
3. Add `--format agent-json` or JSONL streaming instead of only human-oriented formats and raw serialized results in [src/lib.rs](/Users/jaymalave/Desktop/FlickJS/flickjs/packages/linter/src/lib.rs):685. Stream `file_started`, `diagnostic`, `fix_candidate`, `verification_plan`, `summary`.
4. Add metadata to `LintRule`: `autofix_kind`, `cross_file`, `false_positive_risk`, `batch_safe`, `default_verify_steps`, `policy_version`.
5. Support changed-range linting. Agents usually touched a few hunks and want “what did I introduce or fail to fix in these lines?” not “re-explain the whole repo.”
6. Add baseline/debt mode so the agent can distinguish repo debt from regressions introduced by its patch.
7. Group diagnostics by root cause. If one parser issue or one unused import causes ten downstream findings, return a cause tree so the agent fixes the source, not the symptoms.
8. Attach verification contracts to each fix. `prefer-const` may only need relint; future type-sensitive rules may need `tsc`; behavior-changing fixes may need targeted tests. The agent should not guess.
9. Support coordinated fix bundles across files. Single-file replacement ranges will become a bottleneck once rules touch imports, exports, public APIs, or symbol renames.
10. Add fix preconditions: source hash, AST fingerprint, surrounding anchors. That makes patch application safe in iterative or multi-agent runs.
11. Make structured suppression first-class. If no safe code fix exists, return a suppression candidate with required rationale, expiry, owner, and debt score.
12. Add policy traces. Example: `unused_import -> type_reference_counts_as_usage=false -> diagnostic`. This is extremely useful for both debugging and agent planning.
13. Turn repo conventions into explicit policy packs, not prompts. `app`, `library`, `tests`, `scripts`, `generated` should lint differently, and agents perform much better when the linter already knows that.
14. Add “why not autofix” metadata. That tells the agent exactly what extra context it must inspect before editing.
15. Give issues stable lifecycle IDs so an agent can track “same issue still open after patch” without diffing message text.
16. Add a daemon/server mode. Your current product is cold-CLI optimized; agent workflows are warm, iterative, and benefit from a long-lived semantic engine.
17. Record accepted vs rejected fixes over time. Use that to refine confidence, safe-fix thresholds, and default candidate ranking.
18. Expose before/after examples as structured fixtures per rule so the agent can retrieve real repair exemplars instead of hallucinating style.
19. Add blast-radius scoring. A `let` -> `const` rewrite is not the same as deleting an export or changing error handling.
20. Add causality links like `blocked_by`, `must_apply_after`, and `supersedes` so the agent can repair in the right order.

**What I’d Build First**

1. V1: richer diagnostic schema, `agent-json`, safe-fix tiers, changed-range linting, baseline mode.
2. V2: verification contracts, root-cause grouping, structured suppression, stable issue IDs.
3. V3: daemon mode, cross-file fix bundles, policy packs, feedback-driven fix ranking.

If you want, I can turn this into a concrete RFC next: proposed Rust structs, CLI flags, JSON schema, and which existing files to extend first.
