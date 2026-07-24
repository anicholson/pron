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
