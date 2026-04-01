#!/usr/bin/env node

const { execFileSync } = require("child_process");
const { existsSync } = require("fs");
const { join } = require("path");
const { platform, arch } = process;

// Platform-specific binary mapping
const BINARY_MAP = {
  "darwin-arm64": "flint-aarch64-apple-darwin",
  "darwin-x64": "flint-x86_64-apple-darwin",
  "linux-x64": "flint-x86_64-unknown-linux-gnu",
  "linux-arm64": "flint-aarch64-unknown-linux-gnu",
  "win32-x64": "flint-x86_64-pc-windows-msvc.exe",
};

const key = `${platform}-${arch}`;
const binaryName = BINARY_MAP[key];

if (!binaryName) {
  console.error(`Flint does not support ${platform}-${arch} yet.`);
  console.error(
    "Supported: macOS (arm64/x64), Linux (x64/arm64), Windows (x64)",
  );
  process.exit(1);
}

const binaryPath = join(__dirname, "binaries", binaryName);

if (!existsSync(binaryPath)) {
  console.error(`Binary not found: ${binaryPath}`);
  console.error("Try reinstalling: npm install -g @flickjs/lint");
  process.exit(1);
}

// Forward all args to the Rust binary
try {
  execFileSync(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });
} catch (e) {
  process.exit(e.status || 1);
}
