# Test Trees

This file holds the project's test trees. Each tree is an `###` subsection using EARS patterns (`when`/`then`, `while`/`then`, `if`/`then`, `where`/`then`, or bare `then`). Trees are the contract: every expected behaviour and side effect goes here, every tree is verified by a test, every test drives the real implementation.

New trees are composed by the `change` skill.

### Journey: scheduled-dev-tasks (journey: tests/journey_pron.rs)
```
Journey: scheduled-dev-tasks
  when a .prontab with a syntax error is placed and pron -d is started
    then pron refuses to start and reports the parse error
  when the .prontab is fixed with a valid per-minute job and pron -d is started
    then the daemon starts
      then .pron.pid is written
      then .pron.log is appended to with a start event
      when a minute boundary is crossed
        then the job's command runs in the working directory
        then the command's output appears in .pron.log between begin and end markers
      when another minute boundary is crossed
        then the job runs a second time
        and .pron.log shows a second set of begin and end markers
  when pron stop is invoked
    then the daemon receives SIGTERM and .pron.pid is removed and the daemon exits cleanly
```

### System: pron-stop (functional: tests/journey_pron.rs)
```
System: pron-stop
  when the pidfile names a stale pid (process no longer alive)
    then the pidfile is removed and pron stop exits 0
  where /proc is available
    when the pidfile names a reused pid whose cmdline is not pron
      then the pidfile is removed and pron stop exits 0 without signalling
  when the pidfile names a live pron daemon
    then SIGTERM is delivered and the daemon removes the pidfile and exits cleanly
  if the pid still owns no process after 5s
    then pron stop warns and exits 0
```

### Domain: Crontab (src: src/domain/crontab.rs; unit: src/domain/crontab.rs; integration: none; functional: none)
```
Domain: Crontab
  parse
    when the crontab is empty
      then no entries are produced
    when a line starts with #
      then the line is ignored
    when a line is blank
      then the line is ignored
    when a line has five valid fields and a command
      then an entry is produced with the parsed expression and whitespace-collapsed command
    if a line has an invalid field value
      then a parse error is returned naming the line and field
    if a line has fewer than five fields
      then a parse error is returned naming the line
    if a line has exactly five fields and a command
      then the boundary between valid and invalid is pinned
```

### Domain: CronExpr (src: src/domain/cron_expr.rs; unit: src/domain/cron_expr.rs; integration: none; functional: none)
```
Domain: CronExpr
  parse
    if a field value is invalid
      then a parse error is returned naming the field and value
    if a field value is out of range
      then a parse error is returned naming the field and value
      and a parse error is returned for a range with an out-of-range endpoint
    when a field is a step expression
      then only every Nth value in the valid range is set
    when a field is a range expression
      then every value in the inclusive range is set
      and endpoints at the field minimum and maximum are accepted
      and a single-element range is accepted
    when a field is a list expression
      then every listed element is set
    when a field combines ranges and lists
      then the union of all elements is set
    if a step expression has an invalid step
      then a parse error is returned naming the field
  matches
    when called with a tuple that matches a fully numeric expression
      then true is returned
      and false is returned for a one-off minute
      and false is returned for a one-off hour
      and false is returned for a one-off day-of-month
      and false is returned for a one-off month
      and false is returned for a one-off day-of-week
    when called with a minute tuple that matches
      then true is returned
    when called with a minute tuple that does not match
      then false is returned
```

### Use-case: start (src: src/application/start.rs; unit: src/application/start.rs; integration: none; functional: none)
```
Use-case: start
  execute
    when called with a valid crontab and daemon mode
      then the pidfile is written with the current pid
      then a start event is logged
    if called with an invalid crontab
      then a parse error is returned without writing the pidfile
```

### Use-case: scheduler (src: src/application/scheduler.rs; unit: src/application/scheduler.rs; integration: none; functional: none)
```
Use-case: scheduler
  tick
    when the current minute matches an entry
      then the entry's command is run
      then the command output is logged between begin and end markers
    when the current minute does not match any entry
      then no command is run
    when the clock advances across multiple ticks
      then each matching minute fires a command
    if the command fails to spawn
      then the failure is logged
    if the command exits with a non-zero code
      then the exit code is logged
```

### Port: Clock (src: src/application/ports/clock.rs; unit: tests/port_clock.rs; integration: none; functional: none)
```
Port: Clock
  now
    when called
      then the current minute tuple is returned
      and each field is within its valid range
```

### Port: ProcessRunner (src: src/application/ports/process_runner.rs; unit: tests/port_process_runner.rs; integration: none; functional: none)
```
Port: ProcessRunner
  run
    when called with a command
      then the command is executed and stdout returned
      and the exit status is returned as zero on success
```

### Port: Filesystem (src: src/application/ports/filesystem.rs; unit: tests/port_filesystem.rs; integration: none; functional: none)
```
Port: Filesystem
  write_pidfile
    when called with a pid
      then the pid is retrievable
  remove_pidfile
    when called after a write
      then the pid is no longer retrievable
```

### Port: Logger (src: src/application/ports/logger.rs; unit: tests/port_logger.rs; integration: none; functional: none)
```
Port: Logger
  log_start
    when called with a mode, a crontab path, and an entry count
      then a start event containing the mode and entry count is recorded
  log_job
    when called with a command and output
      then begin and end markers with the command and output are recorded
```

### Port: ProcessControl (src: src/application/ports/process_control.rs; unit: tests/port_process_control.rs; integration: none; functional: none)
```
Port: ProcessControl
  current_pid
    when called
      then it returns the expected pid
```
