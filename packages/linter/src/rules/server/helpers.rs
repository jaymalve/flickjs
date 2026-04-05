use crate::rules::LintContext;
use oxc_ast::ast::{Argument, CallExpression, Expression, MemberExpression};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;

const ROUTE_METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "all", "use"];
const ORM_QUERY_METHODS: &[&str] = &[
    "find",
    "findOne",
    "findFirst",
    "findMany",
    "findUnique",
    "first",
    "count",
];
const ORM_MUTATE_METHODS: &[&str] = &[
    "create",
    "createMany",
    "update",
    "updateMany",
    "delete",
    "deleteMany",
    "insert",
    "upsert",
];

pub fn expression_static_name(expression: &Expression<'_>) -> Option<String> {
    let expression = expression.without_parentheses();
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => expression
            .get_member_expr()
            .and_then(member_expression_name),
    }
}

pub fn member_expression_name(member: &MemberExpression<'_>) -> Option<String> {
    let object = expression_static_name(member.object())?;
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}

pub fn is_route_registration(call: &CallExpression<'_>) -> bool {
    let Some(member) = call.callee.get_member_expr() else {
        return false;
    };
    let Some(method) = member.static_property_name() else {
        return false;
    };
    ROUTE_METHODS.contains(&method)
        && route_object_name(member.object()).is_some()
        && call.arguments.iter().any(is_handler_argument)
}

pub fn is_inside_route_handler(ctx: &LintContext, node_id: NodeId) -> bool {
    enclosing_route_handler(ctx, node_id).is_some()
}

pub fn enclosing_route_handler_span(ctx: &LintContext, node_id: NodeId) -> Option<Span> {
    enclosing_route_handler(ctx, node_id).map(|handler| handler.1)
}

pub fn is_orm_query_call(call: &CallExpression<'_>) -> bool {
    call.callee
        .get_member_expr()
        .and_then(|member| member.static_property_name())
        .is_some_and(|name| ORM_QUERY_METHODS.contains(&name))
}

pub fn is_orm_mutate_call(call: &CallExpression<'_>) -> bool {
    call.callee
        .get_member_expr()
        .and_then(|member| member.static_property_name())
        .is_some_and(|name| ORM_MUTATE_METHODS.contains(&name))
}

pub fn is_inside_loop(ctx: &LintContext, node_id: NodeId) -> bool {
    ctx.semantic.nodes().ancestor_kinds(node_id).any(|kind| {
        matches!(
            kind,
            AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
        )
    })
}

pub fn enclosing_function_span(ctx: &LintContext, node_id: NodeId) -> Option<Span> {
    ctx.semantic
        .nodes()
        .ancestor_kinds(node_id)
        .find_map(|kind| match kind {
            AstKind::Function(function) if function.body.is_some() => Some(function.span),
            AstKind::ArrowFunctionExpression(function) => Some(function.span),
            _ => None,
        })
}

pub fn is_async_context(ctx: &LintContext, node_id: NodeId) -> bool {
    ctx.semantic
        .nodes()
        .ancestor_kinds(node_id)
        .any(|kind| match kind {
            AstKind::Function(function) => function.r#async,
            AstKind::ArrowFunctionExpression(function) => function.r#async,
            _ => false,
        })
}

pub fn span_contains(outer: Span, inner: Span) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
}

fn enclosing_route_handler(ctx: &LintContext, node_id: NodeId) -> Option<(NodeId, Span)> {
    for ancestor_id in ctx.semantic.nodes().ancestor_ids(node_id) {
        let span = match ctx.semantic.nodes().kind(ancestor_id) {
            AstKind::Function(function) if function.body.is_some() => function.span,
            AstKind::ArrowFunctionExpression(function) => function.span,
            _ => continue,
        };
        let parent_id = ctx.semantic.nodes().parent_id(ancestor_id);
        let AstKind::CallExpression(call) = ctx.semantic.nodes().kind(parent_id) else {
            continue;
        };
        if is_route_registration(call)
            && call
                .arguments
                .iter()
                .any(|argument| argument.span() == span)
        {
            return Some((parent_id, span));
        }
    }
    None
}

fn route_object_name(expression: &Expression<'_>) -> Option<String> {
    let name = expression_static_name(expression)?;
    if matches!(
        name.as_str(),
        "app" | "router" | "route" | "server" | "fastify"
    ) || name.ends_with(".router")
        || name.ends_with(".routes")
        || name.ends_with(".route")
    {
        Some(name)
    } else {
        None
    }
}

fn is_handler_argument(argument: &Argument<'_>) -> bool {
    matches!(
        argument,
        Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_)
    )
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn has_rule(rule_name: &str, source: &str) -> bool {
        lint_source_for_test_with_project("test.ts", source, &ProjectInfo::test_server())
            .diagnostics
            .into_iter()
            .any(|diagnostic| diagnostic.rule_name == rule_name)
    }

    #[test]
    fn detects_route_handlers() {
        let source = "app.get('/users', async (req, res) => { res.redirect(req.query.url); });\n";
        assert!(has_rule("server/no-unsafe-redirect", source));
    }
}
