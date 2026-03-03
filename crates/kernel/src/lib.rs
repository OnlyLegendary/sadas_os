#![no_std]

pub mod capability;
pub mod ipc;
pub mod scheduler;

use capability::{Capability, CapabilitySpace};
use ipc::Message;
use sadas_logging as logging;
use scheduler::{CpuHint, DeviceTier, Scheduler, Task, TaskId};

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

    pub fn spawn_task(&mut self, id: TaskId, name: &'static str, priority: u8, cpu_hint: CpuHint) {
        self.scheduler.enqueue(Task {
            id,
            name,
            priority,
            cpu_hint,
        });
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

    pub fn interactive_slice_ms(&self) -> u8 {
        self.scheduler.runtime_budget().interactive_slice_ms
    }
}

pub const KMAIN_BOOT_ARG_NONE: u64 = 0;
pub const KMAIN_MESSAGE: &str = "sadas: hello from kernel";

/// Kernel entrypoint calling convention for early boot handoff.
///
/// ABI: `extern "C" fn kmain(boot_info_ptr: u64) -> !`
/// - `boot_info_ptr` is reserved for future boot metadata.
/// - `0` means no boot metadata is provided yet.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub extern "C" fn kmain(boot_info_ptr: u64) -> ! {
    let _ = boot_info_ptr;
    logging::set_backend(logging::serial_com1_backend);
    logging::info(KMAIN_MESSAGE);
    logging::warn("sadas: boot running with minimal metadata");
    logging::error("sadas: demo error channel active");
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
