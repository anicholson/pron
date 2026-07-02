# Mental Model

## Core Domain Identity

- `pron` is a folder-scoped cron: it reads a `.prontab` from its working directory and runs scheduled commands there, in foreground or daemon mode.

## World-to-Code Mapping

## Ubiquitous Language

- crontab — the `.prontab` file; one job per line as `min hour dom mon dow command`.
- entry — a parsed crontab line: a cron expression paired with a command.
- tick — a scheduler wake at a minute boundary.
- pidfile — `.pron.pid`; records the running pid and enforces single-instance.
- foreground mode / daemon mode — pron's two run modes; both run the scheduler loop and log to `.pron.log` and write `.pron.pid`. The `-d` flag currently selects the same code path as foreground (daemonization is not yet implemented).
- stale pidfile — `.pron.pid` whose recorded pid no longer points to a live pron process; `pron stop` detects this via `/proc/{pid}/cmdline` (Linux) and removes the file without signalling.

## Bounded Contexts

## Invariants

- One job runs at a time; the same job never overlaps itself.
- Slots missed while stopped are not re-run.
- The crontab is read on start only; a schedule change requires a restart.

## Decision Rationale

- A hand-written 5-field cron parser; the `cron` crate is 7-field with named days/months and a heavier dependency tree.
- Plain std threads plus `signal-hook`; the workload is sleep-match-spawn-wait, so an async runtime would add footprint without benefit.
- Civil date from epoch computed via Howard Hinnant's algorithm (no `chrono` or `time` dependency); the clock adapter derives dom/mon/dow from raw seconds.

## Temporal View
