#![no_std]

pub mod capability;
pub mod ipc;
pub mod scheduler;

use capability::{Capability, CapabilitySpace};
use ipc::Message;
use scheduler::{DeviceTier, Scheduler, Task, TaskId};

pub struct Kernel {
    scheduler: Scheduler,
    caps: CapabilitySpace,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(),
            caps: CapabilitySpace::new(),
        }
    }

    pub fn set_device_tier(&mut self, tier: DeviceTier) {
        self.scheduler.set_device_tier(tier);
    }

    pub fn spawn_task(&mut self, id: TaskId, name: &'static str, priority: u8) {
        self.scheduler.enqueue(Task { id, name, priority });
    }

    pub fn grant_capability(&mut self, task: TaskId, cap: Capability) {
        self.caps.grant(task, cap);
    }

    pub fn revoke_capability(&mut self, task: TaskId, cap: Capability) {
        self.caps.revoke(task, cap);
    }

    pub fn send_message(&self, message: Message) -> Result<(), ipc::IpcError> {
        if !message.secure_channel
            && self
                .caps
                .has_capability(message.to, Capability::NetworkAccess)
        {
            return Err(ipc::IpcError::InsecureChannelRequired);
        }

        if self.caps.has_capability(message.from, Capability::IpcSend)
            && self.caps.has_capability(message.to, Capability::IpcReceive)
        {
            Ok(())
        } else {
            Err(ipc::IpcError::PermissionDenied)
        }
    }

    pub fn tick(&mut self) -> Option<Task> {
        self.scheduler.next()
    }

    pub fn runtime_budget_hz(&self) -> u16 {
        self.scheduler.runtime_budget().target_hz
    }

    pub fn max_background_tasks(&self) -> u8 {
        self.scheduler.runtime_budget().max_background_tasks
    }
}
