#!/usr/bin/env bash
set -euo pipefail

os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)

case "$os-$arch" in
  linux-x86_64|linux-aarch64|darwin-arm64)
    ;;
  *)
    echo "Unsupported platform: $os-$arch"
    echo "Supported: linux-x86_64, linux-aarch64, darwin-arm64"
    exit 1
    ;;
esac

echo "Platform: $os-$arch"

if [ -n "${PRON_VERSION:-}" ]; then
  version="$PRON_VERSION"
  echo "Using version from PRON_VERSION: $version"
else
  version=$(curl -fsSL https://api.github.com/repos/anicholson/pron/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
  if [ -z "$version" ]; then
    echo "Error: Failed to fetch latest version from GitHub API"
    echo "Set PRON_VERSION=x.y.z to override (e.g., PRON_VERSION=v0.1.0)"
    exit 1
  fi
  echo "Latest version: $version"
fi
