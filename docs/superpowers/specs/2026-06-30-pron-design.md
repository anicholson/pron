# pron — Design Spec

## Purpose

`pron` is a folder-scoped cron: it reads a `.prontab` file from its working directory and runs scheduled commands in that directory. It runs in the foreground or as a daemon, stays unobtrusive and resource-light, and is a good Unix signal citizen — easy to stop cleanly.

The motivating use case is local development, where periodic tasks (for example, a `git pull` to collect what other agents have pushed in a trunk-sync repo) are useful while you're working in a folder but should not run at other times, and should stay scoped as small as possible.

## CLI

Three invocations, all scoped to the current working directory:

- `pron` — run the scheduler in the foreground. Output goes to the terminal's stdout/stderr.
- `pron -d` / `pron --daemon` — daemonize, then run the scheduler.
- `pron stop` — read `.pron.pid` from the cwd, send SIGTERM to the named process, wait for exit, report success or a stale-pidfile warning.

Flags are mutually exclusive: `pron stop` ignores `-d`.

## Filesystem footprint

All files live in the directory pron was started in (the cwd at start):

- `.prontab` — the crontab, read on start.
- `.pron.pid` — pidfile, written on start (both modes), removed on clean shutdown.
- `.pron.log` — log file, appended to in daemon mode only (foreground mode writes to the terminal instead).

These are dotfiles, unobtrusive by convention.

## Crontab format

Classic minimal cron, one job per line:

```
min hour dom mon dow command
```

Five whitespace-separated fields, then the rest of the line is the command (whitespace-collapsed to a single space during parse). Supported field syntax:

- `*` — any value
- `*/N` — every N units
- `N` — a single value
- `N-M` — an inclusive range
- `N,M,K` — a list (commas separate values and ranges)
- Combinations of the above within a field (e.g. `1-5,10`, `*/15,30`)

Explicitly **not** supported (YAGNI): env lines, `@reboot` and other special strings, a user field, named days/months (`Mon`, `Jan`), seconds, years, step values other than `*/N` on the whole field.

`#`-to-end-of-line is a comment. Blank lines are ignored. A line with `#` before the five fields is a full-line comment.

The crontab is read on start only. There is no SIGHUP reload. To change the schedule, stop and restart pron.

## Execution model

- **Resolution**: 1 minute. The scheduler ticks at minute boundaries.
- **Command execution**: each command runs via `/bin/sh -c <command>` with cwd set to the folder pron was started in and the environment inherited from pron (so `PATH`, `cargo`, etc. just work).
- **Sequencing**: jobs run sequentially. The scheduler waits for each job to finish before ticking again.
- **Concurrency gate**: sequential execution is the gate. The same job can never overlap with itself, and cross-job overlap is impossible. No flag, no tracking — it's a property of the execution model.
- **No catch-up**: slots missed while stopped are not re-run. The schedule resumes from the current time.
- **Matching**: on each tick, the scheduler matches the current minute against each entry's expression; a match fires the command.

Jobs are expected to be small by intent; if a job overruns its interval, the schedule slips slightly. This is acceptable for a dev tool and recoverable by keeping tasks small.

## Daemon mode

`pron -d` performs a simple built-in daemonization:

1. Fork once.
2. `setsid` — new session, no controlling terminal.
3. Detach from the terminal.
4. Write the pidfile `.pron.pid`.
5. Redirect stdin → `/dev/null`, stdout/stderr → `.pron.log` (append).
6. Run the scheduler.

Foreground mode (`pron`) also writes `.pron.pid` (so `pron stop` works either way), but output goes to the terminal.

No double-fork. The single-fork + setsid is sufficient for a dev tool that's expected to be stopped via `pron stop` or SIGTERM, not to survive system-level init events.

## Single-instance lock

The pidfile doubles as a single-instance lock. On start, if `.pron.pid` exists and the named process is alive, pron refuses to start and points the user at `pron stop`. A stale pidfile (process no longer alive) is overwritten.

## Logging

One log file, `.pron.log`, appended to in daemon mode (never truncated). In foreground mode, the same content goes to the terminal's stdout/stderr.

Logged events, each a timestamped line:

- pron start (mode, crontab path, number of entries)
- job fired (entry's crontab line or a short identifier)
- job exited with code N (or "exited with signal S" / "failed to spawn: <error>")
- pron shutdown (clean)

Each command's stdout/stderr is written to the log between a `--- begin: <command> ---` and `--- end: <command> ---` marker, interleaved with the event lines. In foreground mode, command output flows straight to pron's stdout/stderr with no markers.

## Signals and shutdown

- **SIGINT / SIGTERM** → stop scheduling new jobs, forward SIGTERM to the in-flight child's process group (if any), wait for it to exit, remove the pidfile, exit 0.
- A second SIGTERM → SIGKILL the child and exit immediately.
- **SIGHUP** → ignored. Survives terminal hangup; no reload (the crontab is read on start only).

`pron stop` reads `.pron.pid`, sends SIGTERM, waits up to a few seconds for the process to exit, then reports success or, if the process is still alive or the pidfile is stale, a warning.

Signal wakeup: the scheduler sleeps in short increments (sub-second) checking an `AtomicBool` shutdown flag set by the signal handler, so shutdown is responsive even mid-sleep. While a child is running, the scheduler polls `try_wait()` in a loop checking the same flag, so a signal during a job aborts it promptly.

## Dependencies

Minimal, by design:

- `signal-hook` — the standard lightweight signal-handling crate, for the shutdown-flag wakeup.
- No async runtime. Plain std threads + `signal-hook` is the lightest correct footprint.
- No cron-parsing crate. The `cron` crate (v0.17.0) is a 7-field parser with named days/months and a chrono-based "upcoming fire times" API — three mismatches with our 5-field, 1-minute-resolution, match-against-now model, plus a non-trivial dependency tree (chrono, winnow, phf, once_cell). A hand-written 5-field parser is ~80 lines of pure domain code, zero deps, trivially testable. See the spike notes in the brainstorming transcript.

## Architecture (hexagonal)

The system is split so the domain is pure and I/O lives in adapters, with each driven port shipping an in-memory twin for tests.

### Domain (pure, no I/O)

- **Cron expression parsing**: `5-field string -> CronExpr`. A `CronExpr` holds five `Field` values, each a bitset of allowed ordinals for that unit.
- **Schedule matching**: `CronExpr` × minute (as a `(min, hour, dom, mon, dow)` tuple) → bool.
- **Crontab parsing + validation**: text → list of `(CronExpr, command)`, with precise errors (which line, which field, what went wrong). Whitespace-collapse the command.

### Driven ports (interfaces)

- `Clock` — now (returns the current minute).
- `Sleeper` — sleep until the next minute boundary (or short-increment sleep checking the shutdown flag).
- `ProcessRunner` — run a command in a given cwd with inherited env → exit status + captured output.
- `ProcessControl` — abort the in-flight child (forward SIGTERM/SIGKILL to its process group).
- `Filesystem` — read crontab, write/remove pidfile, append log.

### Adapters

- `SystemClock`, `ThreadSleeper`, `ShCommandRunner`, `SignalChildControl`, `FsAdapter` — the real implementations against std + signal-hook.
- **In-memory twins** for each port, for use-case and domain tests.

### Use-case

- `Scheduler` — the loop. Ties clock + sleeper + matcher + runner + log + shutdown-flag together. Tested with in-memory ports and a fake clock: fires due jobs, skips non-due, no catch-up, sequential, graceful shutdown.

### Outer shell

- CLI arg parsing, daemonization (fork/setsid/pidfile/stdio redirect), signal handling — the real I/O boundary, tested by spawning the binary.

## Testing layers

- **Domain unit**: expression parse/match; crontab parse + validation errors. Pure, in-memory, fast.
- **Use-case**: scheduler fires due jobs, skips non-due, no catch-up, sequential, graceful shutdown — all with in-memory ports and a fake clock.
- **Port-contract**: each adapter vs its in-memory twin.
- **System**: CLI modes (foreground/daemon/stop), signal handling, pidfile/lock, logging — by spawning the real binary.
- **Journey (outside-in floor)**: drop a `.prontab` with a job that appends to a file every minute; start `pron -d`; watch the file grow; `pron stop`; confirm clean shutdown and pidfile removal.

Rust test framework + tree reporters are bootstrapped by the `setup` skill before implementation.

## Out of scope (YAGNI)

These are explicitly excluded to keep the tool small and honest:

- Concurrent job execution (and therefore per-job concurrency gates beyond sequential execution).
- Alerting hooks (the log line with exit code is the only notification).
- Crontab reload without restart.
- Catch-up for missed slots.
- Named days/months, seconds, years, `@reboot` and other special strings, env lines, a user field.
- A cron-parsing crate.
- An async runtime.
- Double-fork daemonization.
