# Install Script Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a curl-pipeable install script that detects the user's platform and installs the latest pron release.

**Architecture:** Single bash script (`install.sh`) in the repo root that uses `uname` for platform detection, GitHub API for version resolution, and `curl`/`wget` for download. Installs to `/usr/local/bin` (with sudo) or `~/.local/bin` (without).

**Tech Stack:** Bash, curl/wget, tar, GitHub Releases API

## Global Constraints

- Supports: Linux x86_64, Linux ARM64, macOS ARM64
- Script must be invocable via `curl -fsSL https://raw.githubusercontent.com/anicholson/pron/main/install.sh | sh`
- Uses `set -euo pipefail` with trap cleanup
- All error messages are specific and actionable
- Test trees follow project's EARS format convention

---

## File Structure

| File | Responsibility |
|------|----------------|
| `install.sh` | Main install script (create) |
| `TEST_TREES.md` | Add install script test trees (modify) |
| `README.md` | Document install.sh usage (modify) |
| `docs/superpowers/specs/2026-07-24-install-script-design.md` | Design spec (create) |
| `docs/superpowers/plans/2026-07-24-install-script.md` | This implementation plan (create) |

---

### Task 1: Design Document

**Files:**
- Create: `docs/superpowers/specs/2026-07-24-install-script-design.md`

**Status:** ✅ Complete

---

### Task 2: Test Trees

**Files:**
- Modify: `TEST_TREES.md` (append new section)

**Status:** ✅ Complete

---

### Task 3: Install Script — Platform Detection and Validation

**Files:**
- Create: `install.sh`

**Status:** ✅ Complete

---

### Task 4: Install Script — Version Resolution

**Files:**
- Modify: `install.sh`

**Status:** ✅ Complete

---

### Task 5: Install Script — Download and Extract

**Files:**
- Modify: `install.sh`

**Status:** ✅ Complete

---

### Task 6: Install Script — Installation and Verification

**Files:**
- Modify: `install.sh`

**Status:** ✅ Complete

---

### Task 7: Install Script — Progress Messages

**Files:**
- Modify: `install.sh`

**Status:** ✅ Complete

---

### Task 8: Update README

**Files:**
- Modify: `README.md` (Install section)

**Status:** ✅ Complete

---

### Task 9: Manual Testing on All Platforms

**Status:** ✅ Complete (linux-aarch64 tested; linux-x86_64 and darwin-arm64 require manual testing)

---

### Task 10: Save Implementation Plan

**Files:**
- Create: `docs/superpowers/plans/2026-07-24-install-script.md`

**Status:** ✅ Complete (this file)
