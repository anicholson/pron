# pron

`pron` is a folder-scoped cron: it reads a `.prontab` from its working directory and runs scheduled commands there, in foreground or daemon mode. It is developed using the contree test-tree-driven workflow on top of [opencode](https://opencode.ai).

## Install

Requires Rust (edition 2024).

```sh
cargo build
```

## Use

```sh
cargo run          # run the binary
make test          # run all tests (tree-formatted)
make test-lib      # run unit tests only (Domain + Use-case)
make test-mutate   # mutation testing (Domain + Use-case)
```

See `AGENTS.md` for the full per-layer test command reference.

## Configure

Development tooling lives under `.opencode/`:

- `.opencode/opencode.json` — opencode configuration: plugins, permissions, and the instructions file.
- `.opencode/contree.md` — the contree methodology rules; the development contract for this project.
- `.opencode/lib/` — the contree plugin's pure logic and unit tests (`contree-core.ts`).
- `.opencode/plugin/` — opencode plugins: `contree.ts` (drift nudges, mental-model validation, self-care) and `trunk-sync.ts` (session timekeeping).
- `.opencode/skill/` — contree skills: `change`, `tdd`, `sync`, `setup`, `workflow`, `diff-for-humans`, `second-opinion`.

The mental model lives at `MENTAL_MODEL.md` (project root) and the test-tree contract at `TEST_TREES.md` (project root, not yet bootstrapped — see Development workflow).

## Development workflow

This project uses the contree test-tree-driven workflow: expected behaviour is captured as test trees, verified by tests, and driven into the implementation outside-in. See `.opencode/contree.md` for the full rules.

To bootstrap test trees and the test framework, run the `setup` skill.
