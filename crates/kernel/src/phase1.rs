use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelTask {
    pub id: u8,
    pub name: &'static str,
}

#[derive(Debug)]
pub struct PreemptiveScheduler {
    tasks: [KernelTask; 2],
    current: usize,
    tick_count: AtomicU64,
}

impl PreemptiveScheduler {
    pub const fn new(tasks: [KernelTask; 2]) -> Self {
        Self {
            tasks,
            current: 0,
            tick_count: AtomicU64::new(0),
        }
    }

    pub fn init_pic_and_timer(&self) {
        // Phase 1 skeleton for PIC/PIT setup point.
        // Real hardware init is intentionally centralized here.
    }

    pub fn on_timer_interrupt(&mut self) -> KernelTask {
        self.tick_count.fetch_add(1, Ordering::Relaxed);
        self.current = (self.current + 1) % self.tasks.len();
        self.tasks[self.current]
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count.load(Ordering::Relaxed)
    }

    pub fn current_task(&self) -> KernelTask {
        self.tasks[self.current]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_ticks_increment_and_switch_tasks() {
        let mut sched = PreemptiveScheduler::new([
            KernelTask {
                id: 1,
                name: "idle",
            },
            KernelTask {
                id: 2,
                name: "worker",
            },
        ]);

        assert_eq!(sched.tick_count(), 0);
        assert_eq!(sched.current_task().name, "idle");

        let first = sched.on_timer_interrupt();
        assert_eq!(first.name, "worker");
        assert_eq!(sched.tick_count(), 1);

        let second = sched.on_timer_interrupt();
        assert_eq!(second.name, "idle");
        assert_eq!(sched.tick_count(), 2);
    }
}
