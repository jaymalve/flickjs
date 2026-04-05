use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_ast::AstKind;
use oxc_span::GetSpan;

use super::helpers::{
    expression_static_name, is_async_context, is_inside_loop, is_inside_route_handler,
    is_orm_query_call,
};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoNPlusOne),
        Box::new(NoUnboundedQuery),
        Box::new(NoSyncFsInHandler),
        Box::new(NoBlockingCrypto),
        Box::new(NoLargeJsonParseSync),
    ]
}

macro_rules! server_rule {
    ($name:ident, $rule_name:literal, $run_fn:ident) => {
        pub struct $name;

        impl LintRule for $name {
            fn name(&self) -> &'static str {
                $rule_name
            }

            fn default_severity(&self) -> Severity {
                Severity::Warning
            }

            fn applies_to_project(&self, project: &ProjectInfo) -> bool {
                project.has_server_framework()
            }

            fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
                if !ctx.project.has_server_framework() {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

server_rule!(NoNPlusOne, "server/no-n-plus-one", run_no_n_plus_one);
server_rule!(
    NoUnboundedQuery,
    "server/no-unbounded-query",
    run_no_unbounded_query
);
server_rule!(
    NoSyncFsInHandler,
    "server/no-sync-fs-in-handler",
    run_no_sync_fs_in_handler
);
server_rule!(
    NoBlockingCrypto,
    "server/no-blocking-crypto",
    run_no_blocking_crypto
);
server_rule!(
    NoLargeJsonParseSync,
    "server/no-large-json-parse-sync",
    run_no_large_json_parse_sync
);

fn run_no_n_plus_one(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_inside_loop(ctx, node.id())
                || !(is_orm_query_call(call) || is_fetch_like_call(call))
            {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Avoid database or network requests inside loops",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_unbounded_query(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let member = call.callee.get_member_expr()?;
            if !matches!(
                member.static_property_name()?,
                "findMany" | "find" | "findAll"
            ) {
                return None;
            }
            has_query_limit(call).not().then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Add a limit, take, or equivalent bound to large collection queries",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_sync_fs_in_handler(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_inside_route_handler(ctx, node.id()) || !is_sync_fs_call(call) {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Avoid synchronous filesystem calls inside request handlers",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_blocking_crypto(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_blocking_crypto_call(call)
                || !(is_async_context(ctx, node.id()) || is_inside_route_handler(ctx, node.id()))
            {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Avoid blocking crypto APIs in async or request-handling code",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_large_json_parse_sync(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if expression_static_name(&call.callee).as_deref() != Some("JSON.parse") {
                return None;
            }
            let argument = first_argument_expression(call)?;
            contains_request_body(argument).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid synchronously parsing large request bodies on the hot path",
                    argument.span(),
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn is_fetch_like_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee).is_some_and(|name| {
        matches!(
            name.as_str(),
            "fetch" | "axios" | "axios.get" | "axios.post" | "ky" | "ky.get" | "ky.post"
        )
    })
}

fn has_query_limit(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    let Some(argument) = call.arguments.first() else {
        return false;
    };
    let Some(expression) = argument.as_expression() else {
        return false;
    };
    let Expression::ObjectExpression(object) = expression.without_parentheses() else {
        return false;
    };
    object_has_property(object, "take")
        || object_has_property(object, "limit")
        || object_has_property(object, "first")
        || object_has_property(object, "last")
}

fn is_sync_fs_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee).is_some_and(|name| {
        matches!(
            name.as_str(),
            "readFileSync"
                | "writeFileSync"
                | "statSync"
                | "openSync"
                | "readdirSync"
                | "fs.readFileSync"
                | "fs.writeFileSync"
                | "fs.statSync"
                | "fs.openSync"
                | "fs.readdirSync"
        )
    })
}

fn is_blocking_crypto_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee).is_some_and(|name| {
        matches!(
            name.as_str(),
            "pbkdf2Sync"
                | "scryptSync"
                | "crypto.pbkdf2Sync"
                | "crypto.scryptSync"
                | "bcrypt.hashSync"
                | "bcrypt.compareSync"
        )
    })
}

fn first_argument_expression<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<&'a Expression<'a>> {
    call.arguments.first()?.as_expression()
}

fn contains_request_body(expression: &Expression<'_>) -> bool {
    expression_static_name(expression).is_some_and(|name| {
        matches!(name.as_str(), "req.body" | "request.body")
            || name.starts_with("req.body.")
            || name.starts_with("request.body.")
    })
}

fn object_has_property(object: &ObjectExpression<'_>, name: &str) -> bool {
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.key.is_specific_static_name(name)
    })
}

trait BoolExt {
    fn not(self) -> bool;
}

impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project("test.ts", source, &ProjectInfo::test_server())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_n_plus_one_queries() {
        let messages = rule_messages(
            "server/no-n-plus-one",
            "for (const id of ids) { await prisma.user.findMany({ where: { id } }); }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid database or network requests inside loops"]
        );
    }

    #[test]
    fn flags_unbounded_queries() {
        let messages = rule_messages(
            "server/no-unbounded-query",
            "prisma.user.findMany({ where: {} });\n",
        );
        assert_eq!(
            messages,
            vec!["Add a limit, take, or equivalent bound to large collection queries"]
        );
    }

    #[test]
    fn flags_sync_fs_in_handler() {
        let messages = rule_messages(
            "server/no-sync-fs-in-handler",
            "app.get('/file', (req, res) => { fs.readFileSync('a.txt'); });\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid synchronous filesystem calls inside request handlers"]
        );
    }

    #[test]
    fn flags_blocking_crypto() {
        let messages = rule_messages(
            "server/no-blocking-crypto",
            "async function hash() { crypto.pbkdf2Sync(password, salt, 1, 64, 'sha512'); }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid blocking crypto APIs in async or request-handling code"]
        );
    }

    #[test]
    fn flags_request_body_json_parse() {
        let messages = rule_messages("server/no-large-json-parse-sync", "JSON.parse(req.body);\n");
        assert_eq!(
            messages,
            vec!["Avoid synchronously parsing large request bodies on the hot path"]
        );
    }
}
