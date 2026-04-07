use crate::rules::LintContext;
use oxc_ast::ast::{
    CallExpression, ExportDefaultDeclarationKind, Expression, FormalParameters,
    ImportDeclarationSpecifier, Program,
};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use std::collections::HashSet;
use std::sync::LazyLock;

pub static EFFECT_HOOKS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["useEffect", "useLayoutEffect"]));

pub static DEPENDENCY_HOOKS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["useEffect", "useLayoutEffect", "useMemo", "useCallback"]));

pub fn is_hook_call(call: &CallExpression<'_>, names: &HashSet<&'static str>) -> bool {
    match call.callee.without_parentheses() {
        Expression::Identifier(identifier) => names.contains(identifier.name.as_str()),
        expr => expr.get_member_expr().is_some_and(|member| {
            member.object().is_specific_id("React")
                && member
                    .static_property_name()
                    .is_some_and(|name| names.contains(name))
        }),
    }
}

pub fn is_component_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

pub fn is_custom_hook_name(name: &str) -> bool {
    name.starts_with("use")
        && name
            .chars()
            .nth(3)
            .is_some_and(|ch| ch.is_ascii_uppercase())
}

pub fn is_setter_identifier(name: &str) -> bool {
    name.starts_with("set")
        && name
            .chars()
            .nth(3)
            .is_some_and(|ch| ch.is_ascii_uppercase())
}

pub fn has_directive(program: &Program<'_>, directive: &str) -> bool {
    program
        .directives
        .iter()
        .any(|entry| entry.directive.as_str() == directive)
}

pub fn program<'a>(ctx: &'a LintContext<'a>) -> Option<&'a Program<'a>> {
    ctx.semantic
        .nodes()
        .iter()
        .find_map(|node| match node.kind() {
            AstKind::Program(program) => Some(program),
            _ => None,
        })
}

pub fn file_has_directive(ctx: &LintContext, directive: &str) -> bool {
    program(ctx).is_some_and(|program| has_directive(program, directive))
}

pub fn has_import_source(ctx: &LintContext, source: &str) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        matches!(
            node.kind(),
            AstKind::ImportDeclaration(import_decl) if import_decl.source.value.as_str() == source
        )
    })
}

pub fn imports_name_from(ctx: &LintContext, source: &str, name: &str) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return false;
        };
        if import_decl.source.value.as_str() != source {
            return false;
        }
        let Some(specifiers) = &import_decl.specifiers else {
            return false;
        };
        specifiers.iter().any(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                specifier.imported.name() == name
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                name == "default" || specifier.local.name == name
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                name == "*" || specifier.local.name == name
            }
        })
    })
}

pub fn has_jsx_element(ctx: &LintContext, name: &str) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        matches!(
            node.kind(),
            AstKind::JSXOpeningElement(opening) if opening.name.to_string() == name
        )
    })
}

pub fn exports_name(ctx: &LintContext, export_name: &str) -> bool {
    ctx.semantic.nodes().iter().any(|node| match node.kind() {
        AstKind::ExportNamedDeclaration(decl) => {
            decl.specifiers
                .iter()
                .any(|specifier| specifier.exported.name() == export_name)
                || decl
                    .declaration
                    .as_ref()
                    .is_some_and(|declaration| match declaration {
                        oxc_ast::ast::Declaration::VariableDeclaration(decl) => {
                            decl.declarations.iter().any(|declarator| {
                                declarator
                                    .id
                                    .get_identifier_name()
                                    .is_some_and(|name| name == export_name)
                            })
                        }
                        oxc_ast::ast::Declaration::FunctionDeclaration(function) => function
                            .id
                            .as_ref()
                            .is_some_and(|identifier| identifier.name == export_name),
                        oxc_ast::ast::Declaration::ClassDeclaration(class) => class
                            .id
                            .as_ref()
                            .is_some_and(|identifier| identifier.name == export_name),
                        _ => false,
                    })
        }
        AstKind::ExportDefaultDeclaration(decl) => match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
                .id
                .as_ref()
                .is_some_and(|identifier| identifier.name == export_name),
            ExportDefaultDeclarationKind::ClassDeclaration(class) => class
                .id
                .as_ref()
                .is_some_and(|identifier| identifier.name == export_name),
            _ => export_name == "default",
        },
        _ => false,
    })
}

pub fn file_uses_react(ctx: &LintContext) -> bool {
    ctx.semantic.nodes().iter().any(|node| match node.kind() {
        AstKind::JSXElement(_) | AstKind::JSXFragment(_) => true,
        AstKind::ImportDeclaration(import_decl) => matches!(
            import_decl.source.value.as_str(),
            "react" | "react/jsx-runtime" | "react/jsx-dev-runtime" | "next" | "next/react"
        ),
        AstKind::CallExpression(call) => {
            is_hook_call(call, &DEPENDENCY_HOOKS)
                || callee_static_name(&call.callee)
                    .is_some_and(|name| name == "useState" || name == "useReducer")
        }
        _ => false,
    })
}

pub fn count_set_state_calls(node_id: NodeId, ctx: &LintContext) -> usize {
    let node_span = ctx.semantic.nodes().kind(node_id).span();
    ctx.semantic
        .nodes()
        .iter()
        .filter(|node| node.id() != node_id && span_contains(node_span, node.kind().span()))
        .filter(|node| {
            matches!(
                node.kind(),
                AstKind::CallExpression(call) if setter_name(call).is_some()
            )
        })
        .count()
}

pub fn is_inside_loop(node_id: NodeId, ctx: &LintContext) -> bool {
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

pub fn span_contains(outer: Span, inner: Span) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
}

pub fn callee_static_name(callee: &Expression<'_>) -> Option<String> {
    let callee = callee.without_parentheses();
    match callee {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => callee.get_member_expr().and_then(member_expression_name),
    }
}

pub fn member_expression_name(member: &oxc_ast::ast::MemberExpression<'_>) -> Option<String> {
    let object = callee_static_name(member.object())?;
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}

pub fn setter_name(call: &CallExpression<'_>) -> Option<String> {
    let Expression::Identifier(identifier) = call.callee.without_parentheses() else {
        return None;
    };
    let name = identifier.name.as_str();
    is_setter_identifier(name).then(|| name.to_string())
}

pub fn collect_dependency_names(array: &oxc_ast::ast::ArrayExpression<'_>) -> HashSet<String> {
    array
        .elements
        .iter()
        .filter_map(dependency_element_name)
        .collect()
}

pub fn dependency_element_name(
    element: &oxc_ast::ast::ArrayExpressionElement<'_>,
) -> Option<String> {
    match element {
        oxc_ast::ast::ArrayExpressionElement::Identifier(expr) => Some(expr.name.to_string()),
        oxc_ast::ast::ArrayExpressionElement::StaticMemberExpression(expr) => Some(format!(
            "{}.{}",
            callee_static_name(&expr.object)?,
            expr.property.name
        )),
        oxc_ast::ast::ArrayExpressionElement::ComputedMemberExpression(expr) => Some(format!(
            "{}.{}",
            callee_static_name(&expr.object)?,
            expr.static_property_name()?
        )),
        _ => None,
    }
}

pub fn dependency_name(expression: &Expression<'_>) -> Option<String> {
    match expression.without_parentheses() {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => expression
            .get_member_expr()
            .and_then(member_expression_name),
    }
}

pub fn identifiers_in_span(ctx: &LintContext, span: Span) -> HashSet<String> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::IdentifierReference(identifier) if span_contains(span, identifier.span) => {
                Some(identifier.name.to_string())
            }
            _ => None,
        })
        .collect()
}

pub fn collect_param_names(params: &FormalParameters<'_>) -> HashSet<String> {
    let mut names = HashSet::new();
    for param in &params.items {
        collect_binding_names(&param.pattern, &mut names);
    }
    if let Some(rest) = &params.rest {
        collect_binding_names(&rest.rest.argument, &mut names);
    }
    names
}

pub fn collect_binding_names(
    pattern: &oxc_ast::ast::BindingPattern<'_>,
    names: &mut HashSet<String>,
) {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) => {
            names.insert(identifier.name.to_string());
        }
        oxc_ast::ast::BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, names);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        oxc_ast::ast::BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, names);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, names);
            }
        }
        oxc_ast::ast::BindingPattern::AssignmentPattern(pattern) => {
            collect_binding_names(&pattern.left, names);
        }
    }
}
