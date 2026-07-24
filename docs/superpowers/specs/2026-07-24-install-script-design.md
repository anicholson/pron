# Install Script Design

**Date:** 2026-07-24

## Approach

Minimal bash script (~40 lines), Approach 1 — simple implementation with manual testing on each platform during development.

## Script Location

`install.sh` in repo root.

## Invocation

```sh
curl -fsSL https://raw.githubusercontent.com/anicholson/pron/main/install.sh | bash
```

Note: requires `bash` (not `sh`) because the script uses `set -o pipefail`, which is bash-specific. On Ubuntu, `sh` is `dash` and doesn't support `pipefail`.

## Platform Support

- Linux x86_64 (`x86_64-unknown-linux-musl`)
- Linux ARM64 (`aarch64-unknown-linux-musl`)
- macOS ARM64 (`aarch64-apple-darwin`)

## Platform Detection

```sh
os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
```

**Platform mapping:**

| `uname -s` | `uname -m` | Release artifact |
|------------|------------|------------------|
| `Linux` | `x86_64` | `pron-x86_64-unknown-linux-musl.tar.gz` |
| `Linux` | `aarch64` | `pron-aarch64-unknown-linux-musl.tar.gz` |
| `Darwin` | `arm64` | `pron-aarch64-apple-darwin.tar.gz` |

## Version Resolution

Fetch latest release tag from GitHub API:

```sh
version=$(curl -fsSL https://api.github.com/repos/anicholson/pron/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
```

**Fallback:** If API call fails (rate limit, network error), allow user to set `PRON_VERSION` environment variable.

## Download and Installation

1. Download tarball to temp directory
2. Extract binary
3. Install to `/usr/local/bin` (with sudo) or `~/.local/bin` (without)
4. Verify binary is executable and in PATH
5. Cleanup temp directory

## Error Handling

- `set -euo pipefail` to exit on any error
- Trap cleanup on exit to remove temp directory
- Specific error messages for each failure mode:
  - Unsupported platform
  - Missing curl/wget
  - GitHub API failure
  - Download failure
  - Extract failure
  - Install permission denied
  - Verification failure (binary not in PATH)

## Test Trees

Added to `TEST_TREES.md` in EARS format. Manual verification on each platform during development.

## Testing Approach

- **Manual testing:** Run script on Linux x86_64, Linux ARM64, macOS ARM64 during development
- **CI testing:** Future enhancement (not in scope)
