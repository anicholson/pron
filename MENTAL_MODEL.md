# Mental Model

## Core Domain Identity

- `pron` is a folder-scoped cron: it reads a `.prontab` from its working directory and runs scheduled commands there, in foreground or daemon mode.

## World-to-Code Mapping

- `.prontab` is read with a raw `std::fs::read_to_string` call in `main()` and passed straight into `Start::execute` as a string — unlike the pidfile, it has no `Filesystem` port method of its own.
- `pron stop` (`main.rs::do_stop`) is not routed through the `Start`/`Scheduler` use-cases or the `Filesystem`/`ProcessControl` ports at all; it reads `.pron.pid` and calls `libc::kill` directly, re-implementing the pidfile read that `adapters/fs.rs` already provides.
- A cron field (minute/hour/dom/month/dow) is represented as a `u64` bitmask, one bit per legal value (`domain/cron_expr.rs`) — `matches` is five bit tests.
- The daemon readiness handshake (fork, pipe, `"ok"`/`"err: ..."` protocol) lives entirely in `main.rs`, outside any adapter — there's no dedicated port for it; it's launch plumbing, not domain behaviour.

## Ubiquitous Language

- crontab — the `.prontab` file; one job per line as `min hour dom mon dow command`.
- entry — a parsed crontab line: a cron expression paired with a command.
- tick — a scheduler wake at a minute boundary.
- pidfile — `.pron.pid`; records the running pid so `pron stop` can find it, and gates single-instance: `Start::execute` refuses to start while the pidfile names a live pron process (see Invariants).
- foreground mode / daemon mode — pron's two run modes; both write `.pron.pid`. Foreground prints the start event and command output to stdout (no markers, no `.pron.log`); daemon logs both to `.pron.log`. The `-d` flag selects daemon mode: pron forks, the child detaches into its own session (`setsid`, std fds to `/dev/null`), and the launcher exits 0 only once the daemon signals readiness (pidfile written naming the daemon's pid, start event logged) over a pipe — so startup failures stay synchronous and scriptable.
- stale pidfile — `.pron.pid` whose recorded pid no longer points to a live pron process, checked via process-liveness plus `/proc/{pid}/cmdline` (Linux). The same check gates two call sites: `pron stop` removes a stale pidfile without signalling, and `Start::execute` silently replaces a stale pidfile to let a new pron start.
- scheduling clock — `SystemClock` derives minute/hour/dom/month/dow from `SystemTime`'s raw epoch seconds with no timezone conversion; jobs fire on **UTC** wall-clock time, not the host's local time zone.

## Bounded Contexts

## Invariants

- One job runs at a time; the same job never overlaps itself.
- Slots missed while stopped are not re-run.
- The crontab is read on start only; a schedule change requires a restart.
- Single-instance: `Start::execute` refuses to start while the pidfile names a live pron process, leaving the existing process and pidfile untouched (`src/application/start.rs`, `System: single-instance`).

## Decision Rationale

- A hand-written 5-field cron parser; the `cron` crate is 7-field with named days/months and a heavier dependency tree.
- Plain std (no async runtime) plus `signal-hook`, running on a single thread; the workload is sleep-match-spawn-wait, so an async runtime or extra threads would add footprint without benefit.
- Civil date from epoch computed via Howard Hinnant's algorithm (no `chrono` or `time` dependency); the clock adapter derives dom/mon/dow from raw seconds.
- Single fork + `setsid` for daemonization, with a readiness pipe from child to launcher: the launcher reports exactly what the daemon reports at startup, and `pron stop` always has the right pid. Double-fork (to bar reacquiring a controlling terminal) is needless ceremony for a folder-scoped dev tool.

## Temporal View

- The tick loop resyncs to the next `:00` wall-clock boundary from scratch on every iteration (`main.rs::run_scheduler`); a job that overruns its minute causes the loop to jump forward to the next boundary rather than queueing the missed one (see Invariants: missed slots are not re-run).
- No monotonic clock guard: `SystemClock` reads `SystemTime::now()` directly, so an NTP step or manual clock change is reflected immediately in scheduling, for better or worse.
- `.pron.log` grows unbounded for the life of a long-running daemon; there is no rotation or truncation.
