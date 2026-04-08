# @flickjs/scan

Blazing-fast JavaScript and TypeScript linter, powered by Rust.

## Install

```bash
npm install -g @flickjs/scan
```

Or as a dev dependency:

```bash
npm install -D @flickjs/scan
```

## Usage

```bash
# Generate a starter config for the current project
flick-scan init

# Lint current directory
flick-scan check .

# Lint a specific directory
flick-scan check src

# JSON output
flick-scan check . --format json
```

## Configuration

Create a `flick.json` in your project root:

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

With `"detect": true`, Flick Scan auto-enables matching built-in categories for React, Next.js, React
Native, and server-side projects when their frameworks are detected from `package.json`. Explicit
rule settings still win, including `"off"`.

## Supported Platforms

| OS      | Architecture |
|---------|-------------|
| macOS   | arm64, x64  |
| Linux   | x64, arm64  |
| Windows | x64         |

## License

MIT
