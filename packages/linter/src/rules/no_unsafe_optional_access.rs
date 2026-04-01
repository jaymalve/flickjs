use oxc_ast::ast::{Expression, Statement, TSType};
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::node::NodeId;
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
                        if nullable_symbols.contains(&symbol_id)
                            && !is_narrowed_by_if_guard(
                                node.id(),
                                symbol_id,
                                ctx,
                            )
                            && !is_narrowed_by_early_return(
                                node.id(),
                                symbol_id,
                                ctx,
                            )
                            && !is_narrowed_by_logical_and(
                                node.id(),
                                symbol_id,
                                ctx,
                            )
                        {
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

/// Walk ancestors of `node_id` looking for an `IfStatement` whose test
/// references `symbol_id`. If found and the node is inside the consequent
/// (not the test), the symbol is narrowed and access is safe.
fn is_narrowed_by_if_guard(node_id: NodeId, symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let nodes = ctx.semantic.nodes();
    let node_span = nodes.kind(node_id).span();

    let mut current_id = node_id;
    loop {
        let parent_id = nodes.parent_id(current_id);
        if parent_id == current_id {
            break; // reached root
        }
        if let AstKind::IfStatement(if_stmt) = nodes.kind(parent_id) {
            let consequent_span = if_stmt.consequent.span();
            // Make sure the access is inside the consequent, not the test
            if node_span.start >= consequent_span.start
                && node_span.end <= consequent_span.end
                && expr_references_symbol(&if_stmt.test, symbol_id, ctx)
            {
                return true;
            }
        }
        current_id = parent_id;
    }
    false
}

/// Check if a nullable symbol is narrowed by an early-return guard.
/// Detects patterns like `if (!x) return;` or `if (!x) return false;`
/// before the current node, which narrows x to non-null for subsequent code.
fn is_narrowed_by_early_return(node_id: NodeId, symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let nodes = ctx.semantic.nodes();
    let node_span = nodes.kind(node_id).span();

    // Walk up to find the enclosing FunctionBody or BlockStatement
    let mut current_id = node_id;
    loop {
        let parent_id = nodes.parent_id(current_id);
        if parent_id == current_id {
            break;
        }
        let statements = match nodes.kind(parent_id) {
            AstKind::FunctionBody(body) => &body.statements,
            AstKind::BlockStatement(block) => &block.body,
            _ => {
                current_id = parent_id;
                continue;
            }
        };

        // Scan statements in this block for early-return guards before our node
        for stmt in statements.iter() {
            // Only look at statements before our access
            if stmt.span().start >= node_span.start {
                break;
            }
            if is_nullish_guard_with_return(stmt, symbol_id, ctx) {
                return true;
            }
        }

        current_id = parent_id;
    }
    false
}

/// Check if a statement is `if (!symbol) return ...;` or `if (symbol == null) return ...;`
/// i.e. an if-statement that tests the symbol for nullishness and returns early.
fn is_nullish_guard_with_return(stmt: &Statement, symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let if_stmt = match stmt {
        Statement::IfStatement(s) => s,
        _ => return false,
    };

    // The consequent must be a return/throw (early exit)
    if !consequent_is_early_exit(&if_stmt.consequent) {
        return false;
    }

    // The test must be a nullish check on our symbol:
    // - `!symbol`
    // - `symbol == null` / `symbol === null` / `symbol === undefined`
    is_nullish_check(&if_stmt.test, symbol_id, ctx)
}

/// Check if a statement (or block) is an early exit (return/throw).
fn consequent_is_early_exit(stmt: &Statement) -> bool {
    match stmt {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .any(|s| matches!(s, Statement::ReturnStatement(_) | Statement::ThrowStatement(_))),
        _ => false,
    }
}

/// Check if an expression is a nullish check on the given symbol.
/// Handles: `!x`, `x == null`, `x === null`, `x == undefined`, `x === undefined`,
/// and `!x` where x is the symbol.
fn is_nullish_check(expr: &Expression, symbol_id: SymbolId, ctx: &LintContext) -> bool {
    match expr {
        // !symbol or !symbol (negation of truthiness)
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_ast::ast::UnaryOperator::LogicalNot =>
        {
            expr_references_symbol(&unary.argument, symbol_id, ctx)
        }
        // symbol == null, symbol === null, symbol == undefined, symbol === undefined
        Expression::BinaryExpression(binary) => {
            use oxc_ast::ast::BinaryOperator;
            matches!(
                binary.operator,
                BinaryOperator::Equality
                    | BinaryOperator::StrictEquality
            ) && ((expr_references_symbol(&binary.left, symbol_id, ctx)
                && is_null_or_undefined(&binary.right))
                || (is_null_or_undefined(&binary.left)
                    && expr_references_symbol(&binary.right, symbol_id, ctx)))
        }
        _ => false,
    }
}

/// Check if an expression is a `null` or `undefined` literal.
fn is_null_or_undefined(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::NullLiteral(_)
    ) || matches!(expr, Expression::Identifier(id) if id.name == "undefined")
}

/// Check if an expression references a given symbol (directly or through
/// optional chaining / member access).
/// Check if a member expression is on the right side of a `&&` where the left
/// side is a truthiness check on the same symbol.
/// Handles: `x && x.prop`, `!!x && x.prop`, `x != null && x.prop`
fn is_narrowed_by_logical_and(node_id: NodeId, symbol_id: SymbolId, ctx: &LintContext) -> bool {
    let nodes = ctx.semantic.nodes();
    let node_span = nodes.kind(node_id).span();

    let mut current_id = node_id;
    loop {
        let parent_id = nodes.parent_id(current_id);
        if parent_id == current_id {
            break;
        }
        if let AstKind::LogicalExpression(logical) = nodes.kind(parent_id) {
            if logical.operator == oxc_ast::ast::LogicalOperator::And {
                let left_span = logical.left.span();
                // Our node must be on the right side (not the left)
                if node_span.start >= logical.right.span().start
                    && left_span.end <= node_span.start
                    && expr_references_symbol(&logical.left, symbol_id, ctx)
                {
                    return true;
                }
            }
        }
        current_id = parent_id;
    }
    false
}

fn expr_references_symbol(expr: &Expression, symbol_id: SymbolId, ctx: &LintContext) -> bool {
    match expr {
        Expression::Identifier(id) => {
            if let Some(ref_id) = id.reference_id.get() {
                if let Some(sym_id) = ctx.semantic.scoping().get_reference(ref_id).symbol_id() {
                    return sym_id == symbol_id;
                }
            }
            false
        }
        Expression::ChainExpression(chain) => {
            use oxc_ast::ast::ChainElement;
            match &chain.expression {
                ChainElement::StaticMemberExpression(member) => {
                    expr_references_symbol(&member.object, symbol_id, ctx)
                }
                ChainElement::ComputedMemberExpression(member) => {
                    expr_references_symbol(&member.object, symbol_id, ctx)
                }
                ChainElement::CallExpression(call) => {
                    expr_references_symbol(&call.callee, symbol_id, ctx)
                }
                _ => false,
            }
        }
        // Handle non-optional member access: x.prop
        Expression::StaticMemberExpression(member) => {
            expr_references_symbol(&member.object, symbol_id, ctx)
        }
        Expression::ComputedMemberExpression(member) => {
            expr_references_symbol(&member.object, symbol_id, ctx)
        }
        // Handle negation: if (!x)
        Expression::UnaryExpression(unary) => {
            expr_references_symbol(&unary.argument, symbol_id, ctx)
        }
        // Handle binary: if (x != null), if (x !== undefined)
        Expression::BinaryExpression(binary) => {
            expr_references_symbol(&binary.left, symbol_id, ctx)
                || expr_references_symbol(&binary.right, symbol_id, ctx)
        }
        // Handle logical: if (x && x.prop)
        Expression::LogicalExpression(logical) => {
            expr_references_symbol(&logical.left, symbol_id, ctx)
                || expr_references_symbol(&logical.right, symbol_id, ctx)
        }
        // Handle call: if (x?.method())
        Expression::CallExpression(call) => {
            expr_references_symbol(&call.callee, symbol_id, ctx)
        }
        _ => false,
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
    fn ok_with_optional_chain_guard_in_if() {
        // if (x?.prop) { x.prop(...) } should NOT flag
        let msgs = unsafe_access_msgs(
            r#"
            interface Store { connectWallet?: (addr: string) => void }
            const persistStore: Store | null = null;
            if (persistStore?.connectWallet) {
                persistStore.connectWallet("addr");
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_truthiness_guard_in_if() {
        // if (x) { x.prop } should NOT flag
        let msgs = unsafe_access_msgs(
            r#"
            const x: string | null = null;
            if (x) {
                const y = x.length;
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_null_check_guard() {
        // if (x != null) { x.prop } should NOT flag
        let msgs = unsafe_access_msgs(
            r#"
            const x: string | null = null;
            if (x != null) {
                const y = x.length;
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn still_flags_outside_guard() {
        // Access outside the if block should still flag
        let msgs = unsafe_access_msgs(
            r#"
            const x: string | null = null;
            if (x) {
                const y = x.length;
            }
            const z = x.length;
            "#,
        );
        assert_eq!(msgs.len(), 1, "Expected 1 diagnostic but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_logical_and_guard() {
        // if (a && b && persistStore) { persistStore.connectWallet(...) } should NOT flag
        let msgs = unsafe_access_msgs(
            r#"
            interface Store { connectWallet: (addr: string, type: string) => Promise<void> }
            declare const publicKey: string | null;
            declare const keypair: string | null;
            declare const persistStore: Store | null;
            if (publicKey && keypair && persistStore) {
                persistStore.connectWallet(publicKey, "normal-wallet");
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_early_return_negation_guard() {
        // if (!address) return false; narrows address for rest of function
        let msgs = unsafe_access_msgs(
            r#"
            declare function decodeKey(s: string): boolean;
            function isValid(address: string | undefined | null): boolean {
                if (!address) return false;
                if (address.startsWith("G")) {
                    decodeKey(address);
                    return true;
                }
                if (address.startsWith("C")) {
                    decodeKey(address);
                    return true;
                }
                return false;
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_early_return_null_check() {
        // if (x === null) return; narrows x
        let msgs = unsafe_access_msgs(
            r#"
            function foo(x: string | null) {
                if (x === null) return;
                const y = x.length;
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn still_flags_before_early_return() {
        // Access BEFORE the early return guard should still flag
        let msgs = unsafe_access_msgs(
            r#"
            function foo(x: string | null) {
                const y = x.length;
                if (!x) return;
            }
            "#,
        );
        assert_eq!(msgs.len(), 1, "Expected 1 diagnostic but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_inline_logical_and_narrowing() {
        // !!apiKey && apiKey.startsWith(...) — left side narrows right side
        let msgs = unsafe_access_msgs(
            r#"
            function isTestKey(apiKey?: string) {
                return !!apiKey && apiKey.startsWith("pk_test_");
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_simple_logical_and_narrowing() {
        // x && x.prop — simplest form
        let msgs = unsafe_access_msgs(
            r#"
            function foo(x?: string) {
                return x && x.length;
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
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
