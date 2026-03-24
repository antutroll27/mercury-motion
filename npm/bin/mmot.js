#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const binary = process.platform === "win32" ? "mmot.exe" : "mmot";
const binaryPath = path.join(__dirname, binary);

if (!fs.existsSync(binaryPath)) {
  console.error(
    "mercury-motion: binary not found. Run `npm install` or `bun install` to download it.\n" +
      "Alternatively: cargo install mmot"
  );
  process.exit(1);
}

try {
  execFileSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  process.exit(err.status || 1);
}
