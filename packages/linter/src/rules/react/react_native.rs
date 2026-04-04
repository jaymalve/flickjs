use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{Expression, JSXAttribute, JSXAttributeValue, JSXChild, ObjectPropertyKind};
use oxc_ast::AstKind;

use super::helpers::{callee_static_name, span_contains};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoInlineStyles),
        Box::new(NoInlineCallbacks),
        Box::new(NoAnonymousListRender),
        Box::new(NoScrollViewList),
        Box::new(NoRawText),
        Box::new(NoAlert),
        Box::new(NoImageUriLiteral),
        Box::new(RequireKeyExtractor),
    ]
}

macro_rules! rn_rule {
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
                project.has_react_native || project.has_expo
            }

            fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
                if !ctx.project.has_react_native && !ctx.project.has_expo {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

rn_rule!(
    NoInlineStyles,
    "react-native/no-inline-styles",
    run_no_inline_styles
);
rn_rule!(
    NoInlineCallbacks,
    "react-native/no-inline-callbacks",
    run_no_inline_callbacks
);
rn_rule!(
    NoAnonymousListRender,
    "react-native/no-anonymous-list-render",
    run_no_anonymous_list_render
);
rn_rule!(
    NoScrollViewList,
    "react-native/no-scrollview-list",
    run_no_scrollview_list
);
rn_rule!(NoRawText, "react-native/no-raw-text", run_no_raw_text);
rn_rule!(NoAlert, "react-native/no-alert", run_no_alert);
rn_rule!(
    NoImageUriLiteral,
    "react-native/no-image-uri-literal",
    run_no_image_uri_literal
);
rn_rule!(
    RequireKeyExtractor,
    "react-native/require-key-extractor",
    run_require_key_extractor
);

fn run_no_inline_styles(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXAttribute(attribute) = node.kind() else {
                return None;
            };
            if !attribute.is_identifier("style") {
                return None;
            }
            let expression = attribute_expression(attribute)?;
            matches!(
                expression.without_parentheses(),
                Expression::ObjectExpression(_)
            )
            .then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid inline style objects in React Native components",
                    attribute.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_inline_callbacks(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXAttribute(attribute) = node.kind() else {
                return None;
            };
            let name = attribute.name.get_identifier().name.as_str();
            if !matches!(
                name,
                "onPress"
                    | "onLongPress"
                    | "onPressIn"
                    | "onPressOut"
                    | "onChangeText"
                    | "onEndReached"
            ) {
                return None;
            }
            attribute_expression(attribute)
                .filter(|expression| is_function_like_expression(expression))
                .map(|_| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid recreating inline callbacks on React Native surfaces",
                        attribute.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_anonymous_list_render(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if !matches!(
                opening.name.to_string().as_str(),
                "FlatList" | "SectionList"
            ) {
                return None;
            }
            opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| {
                    attribute.is_identifier("renderItem")
                        && attribute_expression(attribute).is_some_and(is_function_like_expression)
                })
                .map(|attribute| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid inline `renderItem` functions on React Native lists",
                        attribute.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_scrollview_list(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXElement(element) = node.kind() else {
                return None;
            };
            if element.opening_element.name.to_string() != "ScrollView" {
                return None;
            }
            ctx.semantic.nodes().iter().any(|candidate| {
                matches!(
                    candidate.kind(),
                    AstKind::CallExpression(call)
                        if span_contains(element.span, call.span)
                            && call
                                .callee
                                .get_member_expr()
                                .and_then(|member| member.static_property_name())
                                == Some("map")
                )
            })
            .then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Prefer `FlatList` or `SectionList` over mapping large collections in `ScrollView`",
                    element.opening_element.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_raw_text(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXElement(element) = node.kind() else {
                return None;
            };
            if !matches!(
                element.opening_element.name.to_string().as_str(),
                "View"
                    | "Pressable"
                    | "TouchableOpacity"
                    | "TouchableHighlight"
                    | "TouchableWithoutFeedback"
                    | "ScrollView"
            ) {
                return None;
            }
            element.children.iter().find_map(|child| match child {
                JSXChild::Text(text) if !text.value.as_str().trim().is_empty() => {
                    Some(ctx.diagnostic(
                        rule_name,
                        "Wrap raw text in a `<Text>` component in React Native",
                        text.span,
                        Severity::Warning,
                    ))
                }
                _ => None,
            })
        })
        .collect()
}

fn run_no_alert(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            callee_static_name(&call.callee)
                .is_some_and(|name| name == "alert" || name == "Alert.alert")
                .then(|| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid blocking alert APIs in React Native UI flows",
                        call.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_image_uri_literal(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if opening.name.to_string() != "Image" {
                return None;
            }
            opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| {
                    attribute.is_identifier("source") && source_has_literal_uri(attribute)
                })
                .map(|attribute| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid hardcoded remote image URIs inside React Native components",
                        attribute.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_require_key_extractor(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if !matches!(
                opening.name.to_string().as_str(),
                "FlatList" | "SectionList"
            ) {
                return None;
            }
            opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .any(|attribute| attribute.is_identifier("keyExtractor"))
                .not()
                .then(|| {
                    ctx.diagnostic(
                        rule_name,
                        "Add `keyExtractor` to React Native list components",
                        opening.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn attribute_expression<'a>(attribute: &'a JSXAttribute<'a>) -> Option<&'a Expression<'a>> {
    let JSXAttributeValue::ExpressionContainer(container) = attribute.value.as_ref()? else {
        return None;
    };
    container.expression.as_expression()
}

fn is_function_like_expression(expression: &Expression<'_>) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

fn source_has_literal_uri(attribute: &JSXAttribute<'_>) -> bool {
    let Some(expression) = attribute_expression(attribute) else {
        return false;
    };
    let Expression::ObjectExpression(object) = expression.without_parentheses() else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        if !property.key.is_specific_static_name("uri") {
            return false;
        }
        match property.value.without_parentheses() {
            Expression::StringLiteral(_) => true,
            Expression::TemplateLiteral(template) => template.expressions.is_empty(),
            _ => false,
        }
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

    fn project() -> ProjectInfo {
        ProjectInfo {
            has_expo: true,
            has_react_native: true,
            ..ProjectInfo::default()
        }
    }

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project("App.tsx", source, &project())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_inline_styles() {
        let messages = rule_messages(
            "react-native/no-inline-styles",
            "export function App() { return <View style={{ padding: 12 }} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid inline style objects in React Native components"]
        );
    }

    #[test]
    fn flags_inline_callbacks() {
        let messages = rule_messages(
            "react-native/no-inline-callbacks",
            "export function App() { return <Pressable onPress={() => doThing()} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid recreating inline callbacks on React Native surfaces"]
        );
    }

    #[test]
    fn flags_anonymous_list_render() {
        let messages = rule_messages(
            "react-native/no-anonymous-list-render",
            "export function App() { return <FlatList renderItem={({ item }) => <Item item={item} />} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid inline `renderItem` functions on React Native lists"]
        );
    }

    #[test]
    fn flags_scrollview_list() {
        let messages = rule_messages(
            "react-native/no-scrollview-list",
            "export function App({ items }) { return <ScrollView>{items.map(item => <Card key={item} />)}</ScrollView>; }\n",
        );
        assert_eq!(
            messages,
            vec![
                "Prefer `FlatList` or `SectionList` over mapping large collections in `ScrollView`"
            ]
        );
    }

    #[test]
    fn flags_raw_text() {
        let messages = rule_messages(
            "react-native/no-raw-text",
            "export function App() { return <View>Hello</View>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Wrap raw text in a `<Text>` component in React Native"]
        );
    }

    #[test]
    fn flags_alert() {
        let messages = rule_messages(
            "react-native/no-alert",
            "export function App() { Alert.alert('Hi'); return null; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid blocking alert APIs in React Native UI flows"]
        );
    }

    #[test]
    fn flags_image_uri_literal() {
        let messages = rule_messages(
            "react-native/no-image-uri-literal",
            "export function App() { return <Image source={{ uri: 'https://cdn.example.com/a.png' }} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid hardcoded remote image URIs inside React Native components"]
        );
    }

    #[test]
    fn flags_missing_key_extractor() {
        let messages = rule_messages(
            "react-native/require-key-extractor",
            "export function App() { return <FlatList data={items} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Add `keyExtractor` to React Native list components"]
        );
    }
}
