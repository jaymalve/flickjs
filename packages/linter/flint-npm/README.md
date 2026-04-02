# @flickjs/lint

Blazing-fast JavaScript and TypeScript linter, powered by Rust.

## Install

```bash
npm install -g @flickjs/lint
```

Or as a dev dependency:

```bash
npm install -D @flickjs/lint
```

## Usage

```bash
# Lint current directory
flint .

# Lint specific files
flint src/index.ts src/utils.ts

# JSON output
flint . --format json
```

## Configuration

Create a `flint.json` in your project root:

```json
{
  "rules": {
    "no-explicit-any": "error",
    "no-unused-vars": "warn",
    "no-console": "off",
    "prefer-const": "error"
  }
}
```

## Supported Platforms

| OS      | Architecture |
|---------|-------------|
| macOS   | arm64, x64  |
| Linux   | x64, arm64  |
| Windows | x64         |

## License

MIT
