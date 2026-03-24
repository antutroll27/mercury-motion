#!/usr/bin/env node

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");

const VERSION = "0.1.0";
const REPO = "antutroll27/mercury-motion";

const PLATFORMS = {
  "linux-x64": { artifact: "mmot-linux-x86_64.tar.gz", binary: "mmot" },
  "darwin-x64": { artifact: "mmot-macos-x86_64.tar.gz", binary: "mmot" },
  "darwin-arm64": { artifact: "mmot-macos-aarch64.tar.gz", binary: "mmot" },
  "win32-x64": { artifact: "mmot-windows-x86_64.zip", binary: "mmot.exe" },
};

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

function downloadFile(url) {
  return new Promise((resolve, reject) => {
    const follow = (url) => {
      https
        .get(url, { headers: { "User-Agent": "mercury-motion-npm" } }, (res) => {
          if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
            follow(res.headers.location);
            return;
          }
          if (res.statusCode !== 200) {
            reject(new Error(`Download failed: HTTP ${res.statusCode}`));
            return;
          }
          const chunks = [];
          res.on("data", (chunk) => chunks.push(chunk));
          res.on("end", () => resolve(Buffer.concat(chunks)));
          res.on("error", reject);
        })
        .on("error", reject);
    };
    follow(url);
  });
}

async function install() {
  const key = getPlatformKey();
  const config = PLATFORMS[key];

  if (!config) {
    console.error(
      `mercury-motion: unsupported platform ${key}\n` +
        `Supported: ${Object.keys(PLATFORMS).join(", ")}\n` +
        `Install from source instead: cargo install mmot`
    );
    process.exit(1);
  }

  const binDir = path.join(__dirname, "bin");
  const binaryPath = path.join(binDir, config.binary);

  // Skip if already installed
  if (fs.existsSync(binaryPath)) {
    return;
  }

  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${config.artifact}`;
  console.log(`mercury-motion: downloading ${config.artifact}...`);

  try {
    const data = await downloadFile(url);

    fs.mkdirSync(binDir, { recursive: true });

    const archivePath = path.join(binDir, config.artifact);
    fs.writeFileSync(archivePath, data);

    // Extract
    if (config.artifact.endsWith(".tar.gz")) {
      execSync(`tar xzf "${archivePath}" -C "${binDir}"`, { stdio: "pipe" });
    } else if (config.artifact.endsWith(".zip")) {
      // Use PowerShell on Windows for zip extraction
      execSync(
        `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${binDir}' -Force"`,
        { stdio: "pipe" }
      );
    }

    // Clean up archive
    fs.unlinkSync(archivePath);

    // Make binary executable on Unix
    if (process.platform !== "win32") {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log("mercury-motion: installed successfully");
  } catch (err) {
    console.error(
      `mercury-motion: failed to download binary\n` +
        `  ${err.message}\n` +
        `  Install from source instead: cargo install mmot`
    );
    process.exit(1);
  }
}

install();
