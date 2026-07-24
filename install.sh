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

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

case "$os-$arch" in
  linux-x86_64)
    artifact="pron-x86_64-unknown-linux-musl.tar.gz"
    ;;
  linux-aarch64)
    artifact="pron-aarch64-unknown-linux-musl.tar.gz"
    ;;
  darwin-arm64)
    artifact="pron-aarch64-apple-darwin.tar.gz"
    ;;
esac

url="https://github.com/anicholson/pron/releases/download/${version}/${artifact}"
echo "Downloading from $url..."

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$url" -o "$tmp/pron.tar.gz"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$tmp/pron.tar.gz" "$url"
else
  echo "Error: curl or wget required"
  exit 1
fi

echo "Extracting..."
tar -xzf "$tmp/pron.tar.gz" -C "$tmp"

if [ -w /usr/local/bin ] || sudo -n true 2>/dev/null; then
  install_dir="/usr/local/bin"
  sudo_cmd="sudo"
else
  install_dir="$HOME/.local/bin"
  mkdir -p "$install_dir"
  sudo_cmd=""
fi

echo "Installing to $install_dir..."
$sudo_cmd mv "$tmp/pron" "$install_dir/pron"
$sudo_cmd chmod +x "$install_dir/pron"

if command -v pron >/dev/null 2>&1; then
  echo "pron installed to $(command -v pron)"
  pron --version 2>/dev/null || echo "pron installed successfully"
else
  echo "Error: pron not found in PATH after installation"
  echo "Add $install_dir to your PATH"
  exit 1
fi
