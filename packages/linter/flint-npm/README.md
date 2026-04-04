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
flint check .

# Lint a specific directory
flint check src

# JSON output
flint check . --format json
```

## Configuration

Create a `flint.json` in your project root:

```json
{
  "detect": true,
  "rules": {
    "no-explicit-any": "error",
    "no-unused-vars": "warn",
    "no-console": "off",
    "prefer-const": "error",
    "react/no-fetch-in-effect": "warn"
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
