use crate::application::ports::clock::Clock;
use crate::application::ports::process_runner::ProcessRunner;
use crate::domain::crontab::Entry;
use crate::domain::cron_expr;

pub struct Scheduler<C: Clock, R: ProcessRunner> {
    clock: C,
    runner: R,
    entries: Vec<Entry>,
}

impl<C: Clock, R: ProcessRunner> Scheduler<C, R> {
    pub fn new(clock: C, runner: R, entries: Vec<Entry>) -> Self {
        Self { clock, runner, entries }
    }

    pub fn tick(&self) {
        let m = self.clock.now();
        for entry in &self.entries {
            if cron_expr::matches(&entry.expr, m.min, m.hour, m.dom, m.mon, m.dow) {
                let _ = self.runner.run(&entry.command);
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
                use crate::application::ports::process_runner::in_memory::InMemoryProcessRunner;
                use crate::application::scheduler::Scheduler;
                use crate::domain::crontab;

                let entries = crontab::parse("* * * * * echo hi\n").unwrap();
                let clock = InMemoryClock::with(0, 0, 1, 1, 0);
                let runner = InMemoryProcessRunner::default();

                let scheduler = Scheduler::new(clock, runner.clone(), entries);
                scheduler.tick();

                let commands = runner.commands.lock().unwrap();
                assert_eq!(commands.len(), 1);
                assert_eq!(commands[0], "echo hi");
            }
        }

        mod when_the_current_minute_does_not_match_any_entry {
            #[test]
            fn then_no_command_is_run() {
                use crate::application::ports::clock::in_memory::InMemoryClock;
                use crate::application::ports::process_runner::in_memory::InMemoryProcessRunner;
                use crate::application::scheduler::Scheduler;
                use crate::domain::crontab;

                let entries = crontab::parse("0 * * * * echo hi\n").unwrap();
                let clock = InMemoryClock::with(30, 0, 1, 1, 0);
                let runner = InMemoryProcessRunner::default();

                let scheduler = Scheduler::new(clock, runner.clone(), entries);
                scheduler.tick();

                let commands = runner.commands.lock().unwrap();
                assert!(commands.is_empty());
            }
        }
    }
}