# Mental Model

## Core Domain Identity

- `pron` is a folder-scoped cron: it reads a `.prontab` from its working directory and runs scheduled commands there, in foreground or daemon mode.

## World-to-Code Mapping

## Ubiquitous Language

- crontab — the `.prontab` file; one job per line as `min hour dom mon dow command`.
- entry — a parsed crontab line: a cron expression paired with a command.
- tick — a scheduler wake at a minute boundary.
- pidfile — `.pron.pid`; records the running pid so `pron stop` can find it. Starting a second pron overwrites it — single-instance is not enforced.
- foreground mode / daemon mode — pron's two run modes; both write `.pron.pid`. Foreground prints the start event and command output to stdout (no markers, no `.pron.log`); daemon logs both to `.pron.log`. The `-d` flag selects daemon mode: pron forks, the child detaches into its own session (`setsid`, std fds to `/dev/null`), and the launcher exits 0 only once the daemon signals readiness (pidfile written naming the daemon's pid, start event logged) over a pipe — so startup failures stay synchronous and scriptable.
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
- Single fork + `setsid` for daemonization, with a readiness pipe from child to launcher: the launcher reports exactly what the daemon reports at startup, and `pron stop` always has the right pid. Double-fork (to bar reacquiring a controlling terminal) is needless ceremony for a folder-scoped dev tool.

## Temporal View
