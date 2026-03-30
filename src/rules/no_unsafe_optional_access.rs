use oxc_ast::ast::TSType;
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::symbol::SymbolId;
use std::collections::HashSet;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

/// Detects property access (`.foo`) on variables whose type annotation
/// includes `null` or `undefined` (e.g., `x: string | null`), without
/// optional chaining (`?.`). TypeScript-only.
pub struct NoUnsafeOptionalAccess;

impl LintRule for NoUnsafeOptionalAccess {
    fn name(&self) -> &'static str {
        "no-unsafe-optional-access"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        if !ctx.source_type.is_typescript() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        // Phase 1: Collect symbols with nullable type annotations
        let mut nullable_symbols: HashSet<SymbolId> = HashSet::new();

        for node in ctx.semantic.nodes().iter() {
            match node.kind() {
                AstKind::VariableDeclarator(decl) => {
                    if let Some(annotation) = &decl.type_annotation {
                        if is_nullable_type(&annotation.type_annotation) {
                            if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &decl.id {
                                nullable_symbols.insert(id.symbol_id());
                            }
                        }
                    }
                }
                AstKind::FormalParameter(param) => {
                    if let Some(annotation) = &param.type_annotation {
                        if is_nullable_type(&annotation.type_annotation) {
                            if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) =
                                &param.pattern
                            {
                                nullable_symbols.insert(id.symbol_id());
                            }
                        }
                    }
                    if param.optional {
                        if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) =
                            &param.pattern
                        {
                            nullable_symbols.insert(id.symbol_id());
                        }
                    }
                }
                _ => {}
            }
        }

        if nullable_symbols.is_empty() {
            return diagnostics;
        }

        // Phase 2: Find member expressions on nullable symbols without optional chaining
        for node in ctx.semantic.nodes().iter() {
            let (object_expr, prop_name, optional, span) = match node.kind() {
                AstKind::StaticMemberExpression(member) => (
                    &member.object,
                    member.property.name.to_string(),
                    member.optional,
                    member.span(),
                ),
                AstKind::ComputedMemberExpression(member) => (
                    &member.object,
                    "[computed]".to_string(),
                    member.optional,
                    member.span(),
                ),
                _ => continue,
            };

            if optional {
                continue;
            }

            if let oxc_ast::ast::Expression::Identifier(id) = object_expr {
                if let Some(reference_id) = id.reference_id.get() {
                    if let Some(symbol_id) =
                        ctx.semantic.scoping().get_reference(reference_id).symbol_id()
                    {
                        if nullable_symbols.contains(&symbol_id) {
                            diagnostics.push(ctx.diagnostic_with_context(
                                self.name(),
                                format!(
                                    "Unsafe access `.{}` on `{}` which may be null or undefined",
                                    prop_name, id.name
                                ),
                                span,
                                Severity::Error,
                                super::RuleOrigin::BuiltIn,
                                Some("MemberExpression".to_string()),
                                Some(id.name.to_string()),
                            ));
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// Check if a type annotation includes null or undefined.
fn is_nullable_type(ts_type: &TSType) -> bool {
    match ts_type {
        TSType::TSNullKeyword(_) => true,
        TSType::TSUndefinedKeyword(_) => true,
        TSType::TSUnionType(union) => union.types.iter().any(|t| is_nullable_type(t)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn unsafe_access_msgs(source: &str) -> Vec<String> {
        lint_source_for_test("test.ts", source)
            .diagnostics
            .into_iter()
            .filter(|d| d.rule_name == "no-unsafe-optional-access")
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn flags_access_on_nullable_variable() {
        let msgs = unsafe_access_msgs(
            "const x: string | null = null;\nconst y = x.length;\n",
        );
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("x"));
        assert!(msgs[0].contains("null or undefined"));
    }

    #[test]
    fn flags_access_on_union_with_undefined() {
        let msgs = unsafe_access_msgs(
            "const x: string | undefined = undefined;\nconst y = x.trim();\n",
        );
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn ok_with_optional_chaining() {
        let msgs = unsafe_access_msgs(
            "const x: string | null = null;\nconst y = x?.length;\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn ok_with_non_nullable_type() {
        let msgs = unsafe_access_msgs(
            "const x: string = 'hello';\nconst y = x.length;\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn flags_optional_param_access() {
        let msgs = unsafe_access_msgs(
            "function foo(x?: string) {\n  const y = x.length;\n}\n",
        );
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn ignores_js_files() {
        let result = crate::rules::lint_source_for_test(
            "test.js",
            "const x = null;\nconst y = x.length;\n",
        );
        let msgs: Vec<_> = result
            .diagnostics
            .into_iter()
            .filter(|d| d.rule_name == "no-unsafe-optional-access")
            .collect();
        assert!(msgs.is_empty());
    }
}
