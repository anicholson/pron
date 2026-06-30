# Mental Model

## Core Domain Identity

- `pron` is a folder-scoped cron: it reads a `.prontab` from its working directory and runs scheduled commands there, in foreground or daemon mode.

## World-to-Code Mapping

## Ubiquitous Language

- crontab — the `.prontab` file; one job per line as `min hour dom mon dow command`.
- entry — a parsed crontab line: a cron expression paired with a command.
- tick — a scheduler wake at a minute boundary.
- pidfile — `.pron.pid`; records the running pid and enforces single-instance.
- foreground mode / daemon mode — pron's two run modes; foreground prints to the terminal, daemon detaches and logs to `.pron.log`.

## Bounded Contexts

## Invariants

## Decision Rationale

## Temporal View
