use crate::application::ports::clock::Clock;
use crate::application::ports::logger::Logger;
use crate::application::ports::process_runner::ProcessRunner;
use crate::domain::crontab::Entry;
use crate::domain::cron_expr;

pub struct Scheduler<C: Clock, R: ProcessRunner, L: Logger> {
    clock: C,
    runner: R,
    logger: L,
    entries: Vec<Entry>,
}

impl<C: Clock, R: ProcessRunner, L: Logger> Scheduler<C, R, L> {
    pub fn new(clock: C, runner: R, logger: L, entries: Vec<Entry>) -> Self {
        Self { clock, runner, logger, entries }
    }

    pub fn tick(&self) {
        let m = self.clock.now();
        for entry in &self.entries {
            if cron_expr::matches(&entry.expr, m.min, m.hour, m.dom, m.mon, m.dow)
                && let Ok(output) = self.runner.run(&entry.command)
            {
                self.logger.log_job(&entry.command, &output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod tick {
        mod when_the_current_minute_matches_an_entry {
            #[test]
            fn then_the_entrys_command_is_run() {
                use crate::application::ports::clock::in_memory::InMemoryClock;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_runner::in_memory::InMemoryProcessRunner;
                use crate::application::scheduler::Scheduler;
                use crate::domain::crontab;

                let entries = crontab::parse("* * * * * echo hi\n").unwrap();
                let clock = InMemoryClock::with(0, 0, 1, 1, 0);
                let runner = InMemoryProcessRunner::default();
                let logger = InMemoryLogger::default();

                let scheduler = Scheduler::new(clock, runner.clone(), logger.clone(), entries);
                scheduler.tick();

                let commands = runner.commands.lock().unwrap();
                assert_eq!(commands.len(), 1);
                assert_eq!(commands[0], "echo hi");
            }

            #[test]
            fn then_the_command_output_is_logged_between_begin_and_end_markers() {
                use crate::application::ports::clock::in_memory::InMemoryClock;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::scheduler::Scheduler;
                use crate::domain::crontab;

                let entries = crontab::parse("* * * * * echo hi\n").unwrap();
                let clock = InMemoryClock::with(0, 0, 1, 1, 0);
                let runner = crate::application::ports::process_runner::in_memory::InMemoryProcessRunner::default();
                let logger = InMemoryLogger::default();

                let scheduler = Scheduler::new(clock, runner, logger.clone(), entries);
                scheduler.tick();

                let events = logger.events.lock().unwrap();
                let combined = events.join("\n");
                assert!(
                    combined.contains("--- begin:"),
                    "log should contain begin marker: {combined}"
                );
                assert!(
                    combined.contains("--- end:"),
                    "log should contain end marker: {combined}"
                );
            }
        }

        mod when_the_current_minute_does_not_match_any_entry {
            #[test]
            fn then_no_command_is_run() {
                use crate::application::ports::clock::in_memory::InMemoryClock;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_runner::in_memory::InMemoryProcessRunner;
                use crate::application::scheduler::Scheduler;
                use crate::domain::crontab;

                let entries = crontab::parse("0 * * * * echo hi\n").unwrap();
                let clock = InMemoryClock::with(30, 0, 1, 1, 0);
                let runner = InMemoryProcessRunner::default();
                let logger = InMemoryLogger::default();

                let scheduler = Scheduler::new(clock, runner.clone(), logger, entries);
                scheduler.tick();

                let commands = runner.commands.lock().unwrap();
                assert!(commands.is_empty());
            }
        }

        mod when_the_clock_advances_across_multiple_ticks {
            #[test]
            fn then_each_matching_minute_fires_a_command() {
                use crate::application::ports::clock::in_memory::InMemoryClock;
                use crate::application::ports::logger::in_memory::InMemoryLogger;
                use crate::application::ports::process_runner::in_memory::InMemoryProcessRunner;
                use crate::application::scheduler::Scheduler;
                use crate::domain::crontab;

                let entries = crontab::parse("0 * * * * echo hi\n").unwrap();
                let clock = InMemoryClock::with(0, 0, 1, 1, 0);
                let runner = InMemoryProcessRunner::default();
                let logger = InMemoryLogger::default();

                let scheduler = Scheduler::new(clock.clone(), runner.clone(), logger, entries);

                scheduler.tick();
                clock.set(1, 0, 1, 1, 0);
                scheduler.tick();
                clock.set(0, 1, 1, 1, 0);
                scheduler.tick();

                let commands = runner.commands.lock().unwrap();
                assert_eq!(commands.len(), 2);
                assert_eq!(commands[0], "echo hi");
                assert_eq!(commands[1], "echo hi");
            }
        }
    }
}