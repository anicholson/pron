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
    then the daemon receives SIGTERM
    then .pron.pid is removed
    then the daemon exits cleanly
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
```

### Domain: CronExpr (src: src/domain/cron_expr.rs; unit: src/domain/cron_expr.rs; integration: none; functional: none)
```
Domain: CronExpr
  parse
    if a field value is invalid
      then a parse error is returned naming the field and value
```

### Use-case: start (src: src/application/start.rs; unit: src/application/start.rs; integration: none; functional: none)
```
Use-case: start
  execute
    when called with a valid crontab and daemon mode
      then the pidfile is written with the current pid
      then a start event is logged
      then the scheduler loop begins
```
