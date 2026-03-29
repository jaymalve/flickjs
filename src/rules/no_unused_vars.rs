use oxc_ast::AstKind;
use oxc_syntax::symbol::SymbolFlags;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct NoUnusedVars;

impl LintRule for NoUnusedVars {
    fn name(&self) -> &'static str {
        "no-unused-vars"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .scoping()
            .symbol_ids()
            .filter_map(|symbol_id| {
                let scoping = ctx.semantic.scoping();
                let flags = scoping.symbol_flags(symbol_id);
                if !should_check_symbol(flags) {
                    return None;
                }
                if has_meaningful_usage(ctx, symbol_id, flags) {
                    return None;
                }
                let name = scoping.symbol_name(symbol_id);
                if name.starts_with('_') {
                    return None;
                }
                let declaration_id = scoping.symbol_declaration(symbol_id);
                if !is_supported_unused_var_declaration(ctx, declaration_id) {
                    return None;
                }

                Some(ctx.diagnostic(
                    self.name(),
                    format!("`{name}` is declared but never used"),
                    scoping.symbol_span(symbol_id),
                    Severity::Error,
                ))
            })
            .collect()
    }
}

fn is_supported_unused_var_declaration(
    ctx: &LintContext,
    declaration_id: oxc_syntax::node::NodeId,
) -> bool {
    let nodes = ctx.semantic.nodes();
    let mut current = Some(declaration_id);
    while let Some(node_id) = current {
        match nodes.kind(node_id) {
            AstKind::VariableDeclarator(_)
            | AstKind::FormalParameter(_)
            | AstKind::FormalParameterRest(_)
            | AstKind::CatchParameter(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_) => return true,
            _ => current = Some(nodes.parent_id(node_id)),
        }
    }
    false
}

fn should_check_symbol(flags: SymbolFlags) -> bool {
    flags.is_variable() || flags.is_catch_variable() || flags.is_import()
}

fn has_meaningful_usage(
    ctx: &LintContext,
    symbol_id: oxc_syntax::symbol::SymbolId,
    flags: SymbolFlags,
) -> bool {
    ctx.semantic
        .scoping()
        .get_resolved_references(symbol_id)
        .any(|reference| reference.is_read() || (flags.is_type_import() && reference.is_type()))
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn unused_var_messages(path: &str, source: &str) -> Vec<String> {
        lint_source_for_test(path, source)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "no-unused-vars")
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_unused_destructured_binding() {
        let messages = unused_var_messages("test.js", "const { used, unused } = props;\nconsole.log(used);\n");
        assert_eq!(messages, vec!["`unused` is declared but never used"]);
    }

    #[test]
    fn flags_unused_parameter() {
        let messages = unused_var_messages("test.js", "function greet(name, unused) { return name; }\n");
        assert_eq!(messages, vec!["`unused` is declared but never used"]);
    }

    #[test]
    fn ignores_underscore_prefixed_parameter() {
        let messages = unused_var_messages("test.js", "function greet(name, _unused) { return name; }\n");
        assert!(messages.is_empty());
    }

    #[test]
    fn flags_unused_catch_binding() {
        let messages = unused_var_messages("test.js", "try { work(); } catch (error) { recover(); }\n");
        assert_eq!(messages, vec!["`error` is declared but never used"]);
    }

    #[test]
    fn keeps_type_only_imports_used_in_type_positions() {
        let messages = unused_var_messages(
            "test.ts",
            "import type { Foo } from './types';\ntype Bar = Foo;\n",
        );
        assert!(messages.is_empty());
    }
}
