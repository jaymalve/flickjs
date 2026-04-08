1. useSearchParams is checking suspense wrapper for anything, if its just a hook, it should check if that hook has a wrapper in this case.
2. react/no-effect-event-handler, how is this checked? lets discuss logic for this.
3. functional setstate, is this needed? because what if someone simply does !isOpen, its the same thing right?
4. react/no-usememo-simple-expr, how is this checked? lets discuss logic for this.
5. no-missing-return false positive resolving. or just remove it? because we do not have any typechecks.

