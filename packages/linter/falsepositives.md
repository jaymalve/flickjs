1. we should not flag peerdeps as unused deps, because the could be used by other deps.
2. files in public should not be flagged as unwanted files
3. alias imports used by `unused-file` and `unused-export` should resolve through `tsconfig.json` or `jsconfig.json` `compilerOptions.baseUrl` and `paths`; bundler-only aliases are still unsupported.