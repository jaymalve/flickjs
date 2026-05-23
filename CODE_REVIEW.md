# Code Review Guidelines

This document defines the code review process and standards for the FlickJS monorepo. All pull requests require at least one approving review before merging.

## Review Goals

Code review exists to:

1. **Catch bugs early** - Identify issues, edge cases, and potential runtime errors before they reach users
2. **Ensure correctness** - Verify that code does what it claims to do, with proper handling of error cases
3. **Maintain quality** - Keep the codebase readable, maintainable, and consistent with existing patterns
4. **Share knowledge** - Help team members understand changes and learn from one another
5. **Improve security** - Identify vulnerabilities, unsafe practices, and compliance issues
6. **Prevent regressions** - Catch unintended side effects and breaking changes

## What to Check

### Correctness

- **Logic is sound**: Does the code do what the PR description claims? Are edge cases handled?
- **No obvious bugs**: Look for off-by-one errors, null/undefined checks, type mismatches, and logical fallacies
- **Error handling**: Are error cases explicitly handled? Do promises reject/throw appropriately?
- **State management**: Are state updates consistent? Is immutability respected where required?

### Readability & Maintainability

- **Clear naming**: Are variables, functions, and classes named descriptively?
- **Code structure**: Is the code organized logically? Are functions/components a reasonable size?
- **Existing patterns**: Does the code follow established patterns in the codebase? Are there unnecessary deviations?
- **Complexity**: Is the code overcomplicated? Can it be simplified without loss of clarity?

### Testing

- **Test coverage**: Are new functions/components tested? Do tests cover the happy path and error cases?
- **Test quality**: Do tests verify behavior, not implementation? Are they maintainable?
- **No skipped tests**: Flag any `.skip()` or `.only()` left in the code

### Types (TypeScript)

- **Type safety**: Are types defined accurately? Avoid `any`, `unknown` without justification
- **Consistency**: Do types align with the rest of the codebase? Are generics used appropriately?
- **No type suppression**: Flag `@ts-ignore` comments without explanation

### Documentation

- **API documentation**: Are public functions/components documented? Do JSDoc comments add value?
- **Complex logic**: Is non-obvious logic explained in comments?
- **README updates**: Does documentation reflect the changes?

### Security

- **No secrets**: Check for hardcoded API keys, tokens, or credentials
- **Input validation**: User input, external data, and untrusted sources validated appropriately
- **Dependency safety**: New dependencies reviewed for known vulnerabilities
- **XSS/Injection**: Template strings, DOM manipulation, and dynamic code generation reviewed for safety

### Performance & Accessibility

- **Performance regressions**: Are there unnecessary re-renders, loops, or expensive operations?
- **Accessibility**: Do UI changes maintain or improve accessibility? (headings, labels, ARIA, keyboard nav)
- **Bundle impact**: Do changes increase bundle size significantly? (especially in shared libraries)

### Scope & Regressions

- **Scope clarity**: Does the change do one thing well, or does it mix concerns?
- **Backwards compatibility**: Are breaking changes intentional and documented?
- **Related files**: Has the reviewer checked related files for consistency?

## Leaving Feedback

### Be constructive and collaborative

- ✅ "Consider using a Set here for O(1) lookup instead of linear search"
- ❌ "This is inefficient"

### Be specific

- ✅ "The `timeout` error isn't caught in the `fetchUser()` call on line 42"
- ❌ "Error handling needs work"

### Distinguish between must-fix and nice-to-have

Use clear language:
- **Must-fix**: "This will cause a runtime error because..."
- **Should-consider**: "We could improve this by..."
- **Nice-to-have**: "Future improvement: consider..."

### Ask questions

- Use questions to encourage thought: "What happens if the API returns null here?"
- Assume good intent: "I'm not familiar with this pattern—can you explain how it works?"

### Approve changes, not people

Comment on the code, not the author. Focus on what can be improved, not who wrote it.

## Approval Criteria

Approve a PR when:

- ✅ No critical correctness issues that would cause runtime errors
- ✅ No unaddressed security vulnerabilities
- ✅ Tests added for new functionality; existing tests pass
- ✅ Code follows established patterns; deviations are justified
- ✅ Types are accurate; no unwarranted `any` usage
- ✅ No obvious performance regressions
- ✅ Feedback has been addressed (must-fix issues resolved, no outstanding blockers)

Request changes when:

- ❌ Logic errors or edge cases not handled
- ❌ Tests missing or failing
- ❌ New dependencies introduced without justification
- ❌ Security or accessibility regressions
- ❌ Breaking changes not documented or discussed

## Review Checklist

Use this checklist as a starting point. Adapt based on package type and change scope.

### Pre-review Setup
- [ ] Read the PR description and understand the intent
- [ ] Check if this is a breaking change
- [ ] Review the diff and related files

### Correctness
- [ ] Logic is sound and handles edge cases
- [ ] Error cases are handled explicitly
- [ ] No obvious null/undefined issues
- [ ] State updates are consistent

### Quality
- [ ] Code follows existing patterns
- [ ] Names are clear and descriptive
- [ ] Functions/components are reasonably sized
- [ ] No unnecessary complexity

### Types & Testing
- [ ] Types are accurate (no loose `any` usage)
- [ ] New code has test coverage
- [ ] Tests verify behavior, not implementation
- [ ] No skipped or debug tests

### Security & Performance
- [ ] No hardcoded secrets or credentials
- [ ] Input validation appropriate
- [ ] No obvious performance regressions
- [ ] Accessibility maintained (for UI changes)

### Final
- [ ] Documentation/README updated if needed
- [ ] All feedback addressed
- [ ] Approve or request changes

## Package-Specific Guidance

### Runtime Package (`@flickjs/runtime`)

- **Reactivity is correct**: Fine-grained updates only affect intended DOM nodes
- **Memory efficiency**: No leaks in effect cleanup or signal subscriptions
- **API stability**: Changes to public API require discussion

### Compiler Package (`@flickjs/compiler`)

- **Babel plugin correctness**: Transformations produce valid JavaScript
- **Edge cases**: JSX edge cases (fragments, conditional rendering, lists) handled
- **Error messages**: Compilation errors are clear and actionable

### Landing Site & CLI Tools

- **User experience**: CLI output is clear; landing site is performant
- **Accessibility**: Forms, buttons, and navigation are accessible
- **Responsive design**: Mobile and desktop layouts work well

## Common Patterns

### Reviewing refactors
- Does behavior remain identical?
- Are tests passing without modification?
- Is the new structure clearer than the old?

### Reviewing performance improvements
- Include before/after metrics where possible
- Check for regressions in other areas
- Consider readability trade-offs

### Reviewing feature additions
- Is the feature necessary and well-scoped?
- Is the API intuitive?
- Is there a clear use case?

## Questions?

If you're unsure whether feedback is appropriate, default to asking politely. If you disagree with feedback, discuss it in the PR or in sync. The goal is a better codebase, not winning arguments.
