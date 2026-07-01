# pron

`pron` is a folder-scoped cron: it reads a `.prontab` from its working directory and runs scheduled commands there, in foreground or daemon mode. It is developed using the contree test-tree-driven workflow on top of [opencode](https://opencode.ai).

## Install

Requires Rust (edition 2024).

```sh
cargo build
```

## Use

Place a `.prontab` in the working directory, one job per line as `min hour dom mon dow command`:

```sh
echo "* * * * * echo hi" > .prontab   # one job per line: min hour dom mon dow command
pron -d                              # run the scheduler loop (writes .pron.pid, logs to .pron.log)
pron stop                            # stop the running pron (SIGTERM) and remove .pron.pid
```

`-d` / `--daemon` selects daemon mode (the label recorded in the start event); the scheduler loop runs in the invoking process and logs to `.pron.log` in both modes.

## Develop

```sh
cargo build        # build the binary
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

The mental model lives at `MENTAL_MODEL.md` (project root) and the test-tree contract at `TEST_TREES.md` (project root).

## Development workflow

This project uses the contree test-tree-driven workflow: expected behaviour is captured as test trees, verified by tests, and driven into the implementation outside-in. See `.opencode/contree.md` for the full rules.

To bootstrap test trees and the test framework, run the `setup` skill.
