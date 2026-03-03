use crate::scheduler::TaskId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    IpcSend,
    IpcReceive,
    MemoryMap,
    DeviceControl,
}

#[derive(Clone, Copy)]
struct Entry {
    task: TaskId,
    cap: Capability,
}

pub struct CapabilitySpace {
    entries: [Option<Entry>; 64],
}

impl CapabilitySpace {
    pub fn new() -> Self {
        Self { entries: [None; 64] }
    }

    pub fn grant(&mut self, task: TaskId, cap: Capability) {
        if let Some(slot) = self.entries.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(Entry { task, cap });
        }
    }

    pub fn has_capability(&self, task: TaskId, cap: Capability) -> bool {
        self.entries
            .iter()
            .flatten()
            .any(|entry| entry.task == task && entry.cap == cap)
    }
}
