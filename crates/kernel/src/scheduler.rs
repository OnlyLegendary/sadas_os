#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub name: &'static str,
    pub priority: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceTier {
    Legacy,
    Balanced,
    Modern,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeBudget {
    pub target_hz: u16,
    pub max_background_tasks: u8,
}

impl DeviceTier {
    pub fn runtime_budget(self) -> RuntimeBudget {
        match self {
            DeviceTier::Legacy => RuntimeBudget {
                target_hz: 30,
                max_background_tasks: 2,
            },
            DeviceTier::Balanced => RuntimeBudget {
                target_hz: 60,
                max_background_tasks: 6,
            },
            DeviceTier::Modern => RuntimeBudget {
                target_hz: 120,
                max_background_tasks: 12,
            },
        }
    }
}

pub struct Scheduler {
    run_queue: [Option<Task>; 128],
    head: usize,
    tail: usize,
    tier: DeviceTier,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            run_queue: [None; 128],
            head: 0,
            tail: 0,
            tier: DeviceTier::Balanced,
        }
    }

    pub fn set_device_tier(&mut self, tier: DeviceTier) {
        self.tier = tier;
    }

    pub fn runtime_budget(&self) -> RuntimeBudget {
        self.tier.runtime_budget()
    }

    pub fn enqueue(&mut self, task: Task) {
        let next_tail = (self.tail + 1) % self.run_queue.len();
        if next_tail != self.head {
            self.run_queue[self.tail] = Some(task);
            self.tail = next_tail;
        }
    }

    pub fn next(&mut self) -> Option<Task> {
        if self.head == self.tail {
            return None;
        }

        let mut best_index = self.head;
        let mut cursor = self.head;

        while cursor != self.tail {
            if let (Some(candidate), Some(best)) =
                (self.run_queue[cursor], self.run_queue[best_index])
            {
                if candidate.priority > best.priority {
                    best_index = cursor;
                }
            }
            cursor = (cursor + 1) % self.run_queue.len();
        }

        let selected = self.run_queue[best_index];
        self.run_queue[best_index] = self.run_queue[self.head];
        self.run_queue[self.head] = None;
        self.head = (self.head + 1) % self.run_queue.len();
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_priority_runs_first() {
        let mut sched = Scheduler::new();
        sched.enqueue(Task {
            id: TaskId(1),
            name: "low",
            priority: 10,
        });
        sched.enqueue(Task {
            id: TaskId(2),
            name: "high",
            priority: 250,
        });
        let task = sched.next().expect("task available");
        assert_eq!(task.name, "high");
    }
}
