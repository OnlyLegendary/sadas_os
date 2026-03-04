use crate::scheduler::TaskId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    IpcSend,
    IpcReceive,
    MemoryMap,
    DeviceControl,
    UiCompose,
    NetworkAccess,
}

#[derive(Clone, Copy)]
struct Entry {
    task: TaskId,
    cap: Capability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityEvent {
    Granted(TaskId, Capability),
    Revoked(TaskId, Capability),
}

pub struct CapabilitySpace {
    entries: [Option<Entry>; 128],
    audit_log: [Option<CapabilityEvent>; 128],
    audit_cursor: usize,
}

impl CapabilitySpace {
    pub fn new() -> Self {
        Self {
            entries: [None; 128],
            audit_log: [None; 128],
            audit_cursor: 0,
        }
    }

    pub fn grant(&mut self, task: TaskId, cap: Capability) {
        if let Some(slot) = self.entries.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(Entry { task, cap });
            self.log(CapabilityEvent::Granted(task, cap));
        }
    }

    pub fn revoke(&mut self, task: TaskId, cap: Capability) {
        if let Some(slot) = self.entries.iter_mut().find(|slot| {
            slot.map(|e| e.task == task && e.cap == cap)
                .unwrap_or(false)
        }) {
            *slot = None;
            self.log(CapabilityEvent::Revoked(task, cap));
        }
    }

    pub fn has_capability(&self, task: TaskId, cap: Capability) -> bool {
        self.entries
            .iter()
            .flatten()
            .any(|entry| entry.task == task && entry.cap == cap)
    }

    pub fn recent_events(&self) -> &[Option<CapabilityEvent>; 128] {
        &self.audit_log
    }

    fn log(&mut self, event: CapabilityEvent) {
        self.audit_log[self.audit_cursor % self.audit_log.len()] = Some(event);
        self.audit_cursor = self.audit_cursor.wrapping_add(1);
    }
}
