import * as t from "@babel/types"; // jsx parser
import { PluginObj, NodePath } from "@babel/core";

// Detection utilities for list expressions
function isMapCallExpression(expr: t.Expression): boolean {
  return (
    t.isCallExpression(expr) &&
    t.isMemberExpression(expr.callee) &&
    t.isIdentifier(expr.callee.property) &&
    expr.callee.property.name === "map"
  );
}

function isArrayLiteral(expr: t.Expression): boolean {
  return t.isArrayExpression(expr);
}

function isListExpression(expr: t.Expression): boolean {
  return isMapCallExpression(expr) || isArrayLiteral(expr);
}

export default function flickJSX(): PluginObj {
  let needsEffectImport = false;
  let needsRenderListImport = false;

  return {
    visitor: {
      Program: {
        enter() {
          // Reset state for each file
          needsEffectImport = false;
          needsRenderListImport = false;
        },
        exit(path: NodePath<t.Program>) {
          if (!needsEffectImport && !needsRenderListImport) return;

          // Find existing @flickjs/runtime import
          const existingImport = path.node.body.find(
            (node): node is t.ImportDeclaration =>
              t.isImportDeclaration(node) &&
              node.source.value === "@flickjs/runtime"
          );

          // Check which imports already exist
          const hasEffectImport = existingImport?.specifiers.some(
            (spec) =>
              t.isImportSpecifier(spec) &&
              t.isIdentifier(spec.imported) &&
              spec.imported.name === "effect"
          );

          const hasRenderListImport = existingImport?.specifiers.some(
            (spec) =>
              t.isImportSpecifier(spec) &&
              t.isIdentifier(spec.imported) &&
              spec.imported.name === "renderList"
          );

          // Build list of specifiers to add
          const specsToAdd: t.ImportSpecifier[] = [];

          if (needsEffectImport && !hasEffectImport) {
            specsToAdd.push(
              t.importSpecifier(t.identifier("effect"), t.identifier("effect"))
            );
          }

          if (needsRenderListImport && !hasRenderListImport) {
            specsToAdd.push(
              t.importSpecifier(
                t.identifier("renderList"),
                t.identifier("renderList")
              )
            );
          }

          if (specsToAdd.length === 0) return;

          if (existingImport) {
            // Add to existing import
            existingImport.specifiers.push(...specsToAdd);
          } else {
            // Create new import
            const importDecl = t.importDeclaration(
              specsToAdd,
              t.stringLiteral("@flickjs/runtime")
            );
            path.node.body.unshift(importDecl);
          }
        },
      },
      JSXElement: {
        exit(path: NodePath<t.JSXElement>) {
          const opening = path.node.openingElement;
          const tag = opening.name;

          if (!t.isJSXIdentifier(tag)) return;

          const tagName = tag.name;
          const isComponent = tagName[0] === tagName[0].toUpperCase();

          if (isComponent) {
            // Handle component calls: <Link href="/" /> -> Link({ href: "/", children: ... })
            const props: t.ObjectProperty[] = [];

            // Process JSX attributes as props
            opening.attributes.forEach((attr) => {
              if (!t.isJSXAttribute(attr)) return;
              if (!t.isJSXIdentifier(attr.name)) return;

              const attrName = attr.name.name;
              let attrValue: t.Expression;

              if (t.isJSXExpressionContainer(attr.value)) {
                attrValue = t.cloneNode(attr.value.expression as t.Expression);
              } else if (t.isStringLiteral(attr.value)) {
                attrValue = t.cloneNode(attr.value);
              } else if (attr.value === null) {
                attrValue = t.booleanLiteral(true);
              } else {
                return;
              }

              props.push(t.objectProperty(t.identifier(attrName), attrValue));
            });

            // Process children
            const childPaths = path.get("children");
            const childNodes: t.Expression[] = [];

            childPaths.forEach((childPath) => {
              const child = childPath.node;

              if (t.isJSXText(child)) {
                const trimmed = child.value.trim();
                if (trimmed) {
                  // Normalize whitespace in text content
                  const normalized = child.value.replace(/\s+/g, " ").trim();
                  childNodes.push(t.stringLiteral(normalized));
                }
              } else if (t.isJSXExpressionContainer(child)) {
                if (!t.isJSXEmptyExpression(child.expression)) {
                  childNodes.push(t.cloneNode(child.expression as t.Expression));
                }
              } else if (t.isCallExpression(child)) {
                // Already transformed nested JSX element
                childNodes.push(t.cloneNode(child));
              }
            });

            // Add children prop if there are any
            if (childNodes.length > 0) {
              let childrenValue: t.Expression;

              if (childNodes.length === 1) {
                childrenValue = childNodes[0];
              } else {
                childrenValue = t.arrayExpression(childNodes);
              }

              // Wrap Suspense children in arrow function for deferred execution
              // This ensures children are evaluated inside Suspense's effect,
              // when the suspense context is on the stack
              if (tagName === "Suspense") {
                childrenValue = t.arrowFunctionExpression([], childrenValue);
              }

              props.push(
                t.objectProperty(t.identifier("children"), childrenValue)
              );
            }

            // Call the component: Component({ ...props })
            path.replaceWith(
              t.callExpression(t.identifier(tagName), [
                t.objectExpression(props),
              ])
            );
          } else {
            // Handle DOM elements: <div /> -> document.createElement("div")
            const el = path.scope.generateUidIdentifier("el");

            const statements: t.Statement[] = [
              t.variableDeclaration("const", [
                t.variableDeclarator(
                  el,
                  t.callExpression(
                    t.memberExpression(
                      t.identifier("document"),
                      t.identifier("createElement")
                    ),
                    [t.stringLiteral(tagName)]
                  )
                ),
              ]),
            ];

            // Process JSX attributes
            opening.attributes.forEach((attr) => {
              if (!t.isJSXAttribute(attr)) return;
              if (!t.isJSXIdentifier(attr.name)) return;

              const attrName = attr.name.name;
              let attrValue: t.Expression;

              if (t.isJSXExpressionContainer(attr.value)) {
                // Expression value: onclick={() => ...}
                attrValue = t.cloneNode(attr.value.expression as t.Expression);
              } else if (t.isStringLiteral(attr.value)) {
                // String value: class="foo"
                attrValue = t.cloneNode(attr.value);
              } else if (attr.value === null) {
                // Boolean attribute: disabled
                attrValue = t.booleanLiteral(true);
              } else {
                return;
              }

              // For event handlers (onclick, onchange, etc.) and properties, use direct assignment
              // el.onclick = handler
              // Special case: "class" attribute maps to "className" property in DOM
              const propName = attrName === "class" ? "className" : attrName;

              statements.push(
                t.expressionStatement(
                  t.assignmentExpression(
                    "=",
                    t.memberExpression(t.cloneNode(el), t.identifier(propName)),
                    attrValue
                  )
                )
              );
            });

            // Use path.get('children') to get the transformed children
            const childPaths = path.get("children");
            childPaths.forEach((childPath) => {
              const child = childPath.node;

              if (t.isJSXText(child)) {
                // Skip whitespace-only text nodes
                const trimmed = child.value.trim();
                if (trimmed) {
                  statements.push(
                    t.expressionStatement(
                      t.callExpression(
                        t.memberExpression(
                          t.cloneNode(el),
                          t.identifier("append")
                        ),
                        [t.stringLiteral(child.value)]
                      )
                    )
                  );
                }
              } else if (t.isJSXExpressionContainer(child)) {
                // Skip empty expressions
                if (t.isJSXEmptyExpression(child.expression)) return;

                const expr = child.expression as t.Expression;

                if (isListExpression(expr)) {
                  // LIST RENDERING PATH
                  needsRenderListImport = true;
                  needsEffectImport = true;

                  const anchor = path.scope.generateUidIdentifier("anchor");

                  // Create comment anchor: const _anchor = document.createComment("list")
                  statements.push(
                    t.variableDeclaration("const", [
                      t.variableDeclarator(
                        anchor,
                        t.callExpression(
                          t.memberExpression(
                            t.identifier("document"),
                            t.identifier("createComment")
                          ),
                          [t.stringLiteral("list")]
                        )
                      ),
                    ])
                  );

                  // Append anchor to parent: el.append(_anchor)
                  statements.push(
                    t.expressionStatement(
                      t.callExpression(
                        t.memberExpression(
                          t.cloneNode(el),
                          t.identifier("append")
                        ),
                        [t.cloneNode(anchor)]
                      )
                    )
                  );

                  // Generate renderList call
                  if (isMapCallExpression(expr)) {
                    const callExpr = expr as t.CallExpression;
                    const memberExpr = callExpr.callee as t.MemberExpression;
                    const arrayExpr = memberExpr.object;
                    const mapCallback = callExpr.arguments[0] as t.Expression;

                    // Extract key from the JSX element inside the map callback
                    let keyExtractor: t.Expression | null = null;

                    if (
                      t.isArrowFunctionExpression(mapCallback) ||
                      t.isFunctionExpression(mapCallback)
                    ) {
                      const callbackBody = t.isArrowFunctionExpression(
                        mapCallback
                      )
                        ? mapCallback.body
                        : mapCallback.body;

                      // Find the returned JSX element to extract key prop
                      let jsxElement: t.JSXElement | null = null;

                      if (t.isJSXElement(callbackBody)) {
                        jsxElement = callbackBody;
                      } else if (
                        t.isBlockStatement(callbackBody) &&
                        callbackBody.body.length > 0
                      ) {
                        const lastStmt =
                          callbackBody.body[callbackBody.body.length - 1];
                        if (
                          t.isReturnStatement(lastStmt) &&
                          t.isJSXElement(lastStmt.argument)
                        ) {
                          jsxElement = lastStmt.argument;
                        }
                      }

                      // Extract key prop if present
                      if (jsxElement) {
                        const keyAttr = jsxElement.openingElement.attributes.find(
                          (attr): attr is t.JSXAttribute =>
                            t.isJSXAttribute(attr) &&
                            t.isJSXIdentifier(attr.name) &&
                            attr.name.name === "key"
                        );

                        if (
                          keyAttr &&
                          t.isJSXExpressionContainer(keyAttr.value)
                        ) {
                          const keyExpr = keyAttr.value
                            .expression as t.Expression;
                          const params = t.isArrowFunctionExpression(
                            mapCallback
                          )
                            ? mapCallback.params
                            : (mapCallback as t.FunctionExpression).params;

                          // Create key extractor function with same params
                          keyExtractor = t.arrowFunctionExpression(
                            params.map((p) => t.cloneNode(p)),
                            t.cloneNode(keyExpr)
                          );

                          // Remove key prop from JSX element (it's not a DOM attribute)
                          jsxElement.openingElement.attributes =
                            jsxElement.openingElement.attributes.filter(
                              (attr) =>
                                !(
                                  t.isJSXAttribute(attr) &&
                                  t.isJSXIdentifier(attr.name) &&
                                  attr.name.name === "key"
                                )
                            );
                        }
                      }
                    }

                    // Build renderList arguments
                    const renderListArgs: t.Expression[] = [
                      t.cloneNode(el),
                      t.cloneNode(anchor),
                      t.arrowFunctionExpression([], t.cloneNode(arrayExpr)),
                      t.cloneNode(mapCallback),
                    ];

                    if (keyExtractor) {
                      renderListArgs.push(keyExtractor);
                    }

                    // renderList(el, anchor, () => array, mapFn, keyFn?)
                    statements.push(
                      t.expressionStatement(
                        t.callExpression(
                          t.identifier("renderList"),
                          renderListArgs
                        )
                      )
                    );
                  } else if (isArrayLiteral(expr)) {
                    // For array literals like {[<li>a</li>, <li>b</li>]}
                    // Use renderList with identity accessor
                    const arrayExpr = expr as t.ArrayExpression;

                    statements.push(
                      t.expressionStatement(
                        t.callExpression(t.identifier("renderList"), [
                          t.cloneNode(el),
                          t.cloneNode(anchor),
                          t.arrowFunctionExpression(
                            [],
                            t.cloneNode(arrayExpr)
                          ),
                          t.arrowFunctionExpression(
                            [t.identifier("item")],
                            t.identifier("item")
                          ),
                        ])
                      )
                    );
                  }
                } else {
                  // TEXT NODE PATH (original behavior)
                  needsEffectImport = true;

                  const text = path.scope.generateUidIdentifier("text");

                  statements.push(
                    t.variableDeclaration("const", [
                      t.variableDeclarator(
                        text,
                        t.callExpression(
                          t.memberExpression(
                            t.identifier("document"),
                            t.identifier("createTextNode")
                          ),
                          [t.stringLiteral("")]
                        )
                      ),
                    ])
                  );

                  statements.push(
                    t.expressionStatement(
                      t.callExpression(
                        t.memberExpression(
                          t.cloneNode(el),
                          t.identifier("append")
                        ),
                        [t.cloneNode(text)]
                      )
                    )
                  );

                  statements.push(
                    t.expressionStatement(
                      t.callExpression(t.identifier("effect"), [
                        t.arrowFunctionExpression(
                          [],
                          t.assignmentExpression(
                            "=",
                            t.memberExpression(
                              t.cloneNode(text),
                              t.identifier("data")
                            ),
                            t.cloneNode(expr)
                          )
                        ),
                      ])
                    )
                  );
                }
              } else if (t.isCallExpression(child)) {
                // This is a transformed nested JSX element (already an IIFE)
                statements.push(
                  t.expressionStatement(
                    t.callExpression(
                      t.memberExpression(
                        t.cloneNode(el),
                        t.identifier("append")
                      ),
                      [t.cloneNode(child)]
                    )
                  )
                );
              }
            });

            statements.push(t.returnStatement(t.cloneNode(el)));

            path.replaceWith(
              t.callExpression(
                t.arrowFunctionExpression([], t.blockStatement(statements)),
                []
              )
            );
          }
        },
      },
    },
  };
}
