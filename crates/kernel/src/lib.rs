#![no_std]

pub mod capability;
pub mod ipc;
pub mod phase1;
pub mod scheduler;

use capability::{Capability, CapabilitySpace};
use ipc::Message;
use phase1::{KernelTask, PreemptiveScheduler};
use sadas_boot_protocol::BootInfo;
use sadas_console as console;
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
pub const PHASE1_BOOT_LINES: [&str; 7] = [
    "[sadas][INFO] sadas: hello from kernel",
    "[sadas][WARN] sadas: pic/pit timer initialized",
    "[sadas][INFO] sadas: tick -> task worker",
    "[sadas][INFO] sadas: tick -> task idle",
    "[sadas][INFO] sadas: tick -> task worker",
    "[sadas][INFO] sadas: tick -> task idle",
    "[sadas][INFO] sadas: phase1 scheduler loop entered",
];

/// Kernel entrypoint calling convention for early boot handoff.
///
/// ABI: `extern "C" fn kmain(boot_info_ptr: u64) -> !`
/// - `boot_info_ptr` is reserved for future boot metadata.
/// - `0` means no boot metadata is provided yet.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub extern "C" fn kmain(boot_info_ptr: u64) -> ! {
    let _ = boot_info_ptr;
    console::init_serial();
    logging::set_backend(console::logging_backend);
    console::info!("{}", KMAIN_MESSAGE);

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
    sched.init_pic_and_timer();
    console::warn!("sadas: pic/pit timer initialized");

    for _ in 0..4 {
        if let Some(task) = sched.on_timer_interrupt() {
            if task.id == 1 {
                logging::info("sadas: tick -> task idle");
            } else {
                logging::info("sadas: tick -> task worker");
            }
        }
    }

    enable_interrupts();
    console::info!("sadas: phase1 scheduler loop entered");
    log_scheduler_diagnostics(&sched);

    loop {
        halt_cpu();

        if let Some(task) = sched.on_timer_interrupt() {
            let ticks = sched.tick_count();
            if ticks % 100 == 0 {
                console::info!("sadas: scheduler heartbeat");
                logging::debug(task.name);
                log_scheduler_diagnostics(&sched);
            }
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub extern "C" fn kmain_boot_info(boot_info_ptr: *const BootInfo) -> ! {
    if boot_info_ptr.is_null() {
        kmain(KMAIN_BOOT_ARG_NONE);
    }

    let boot_info = unsafe { &*boot_info_ptr };
    console::init_serial();
    console::init_framebuffer(boot_info.framebuffer);

    // Phase 1: use stable handoff ABI and fall back to core loop.
    kmain(KMAIN_BOOT_ARG_NONE)
}

#[cfg(all(not(test), any(target_os = "none", target_os = "uefi")))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    console::init_serial();
    logging::set_backend(console::logging_backend);
    console::error!("sadas: kernel panic");

    if let Some(location) = info.location() {
        console::error!(
            "panic at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    if let Some(message) = info.message().as_str() {
        console::error!("message: {}", message);
    }
    console::error!("backtrace-ish: frame0=kmain frame1=interrupt_or_boot frame2=panic");

    loop {
        halt_cpu();
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn enable_interrupts() {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn halt_cpu() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn halt_cpu() {}

fn log_scheduler_diagnostics(sched: &PreemptiveScheduler) {
    debug_u64(
        "sadas: diag current_task_id",
        sched.current_task().id as u64,
    );
    logging::debug(sched.current_task().name);
    debug_u64("sadas: diag run_queue_len", sched.run_queue_len() as u64);
    if interrupts_enabled() {
        logging::debug("sadas: diag interrupts_enabled=true");
    } else {
        logging::debug("sadas: diag interrupts_enabled=false");
    }
    debug_u64("sadas: diag tick_count", sched.tick_count());
    if !sched.is_initialized() {
        logging::warn("sadas: scheduler not initialized");
    }
}

fn debug_u64(label: &str, value: u64) {
    logging::debug(label);

    let mut digits = [0u8; 20];
    let mut n = value;
    let mut i = digits.len();

    if n == 0 {
        i -= 1;
        digits[i] = b'0';
    } else {
        while n > 0 {
            i -= 1;
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }

    if let Ok(text) = core::str::from_utf8(&digits[i..]) {
        logging::debug(text);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn interrupts_enabled() -> bool {
    let rflags: usize;
    unsafe {
        core::arch::asm!("pushfq", "pop {}", out(reg) rflags, options(nomem, preserves_flags));
    }
    (rflags & (1 << 9)) != 0
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn interrupts_enabled() -> bool {
    false
}
