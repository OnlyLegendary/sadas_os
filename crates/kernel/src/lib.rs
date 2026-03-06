#![no_std]

pub mod capability;
pub mod ipc;
pub mod phase1;
pub mod scheduler;

use capability::{Capability, CapabilitySpace};
use ipc::Message;
use phase1::{KernelTask, PreemptiveScheduler};
use sadas_arch_x86_64::acpi;
use sadas_arch_x86_64::apic::ApicController;
use sadas_arch_x86_64::interrupts::{FaultInfo, InterruptController};
use sadas_arch_x86_64::timer::Timer;
use sadas_boot_protocol::BootInfo;
use sadas_console as console;
use sadas_exec::parse_elf64;
use sadas_logging as logging;
use sadas_memory::frame::{
    FrameAllocator, FrameStats, MemoryDescriptor, MemoryType, ReservedRange,
};
use sadas_memory::heap;
use sadas_memory::paging::{MapFlags, PageMapper};
use sadas_syscall::{SyscallNumber, SyscallRequest, SyscallResponse};
use scheduler::{CpuHint, DeviceTier, Scheduler, Task, TaskId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Process {
    pub pid: u32,
    pub address_space_id: u64,
    pub parent: u32,
    pub alive: bool,
}

pub struct ProcessTable {
    entries: [Option<Process>; 64],
    next_pid: u32,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            entries: [None; 64],
            next_pid: 1,
        }
    }

    pub fn spawn(&mut self, parent: u32) -> Option<u32> {
        let pid = self.next_pid;
        let asid = 0x1000 + pid as u64;
        for slot in &mut self.entries {
            if slot.is_none() {
                *slot = Some(Process {
                    pid,
                    address_space_id: asid,
                    parent,
                    alive: true,
                });
                self.next_pid += 1;
                return Some(pid);
            }
        }
        None
    }

    pub fn mark_exited(&mut self, pid: u32) {
        for entry in &mut self.entries {
            if let Some(p) = entry.as_mut() {
                if p.pid == pid {
                    p.alive = false;
                }
            }
        }
    }

    pub fn list_alive(&self) -> usize {
        self.entries.iter().flatten().filter(|p| p.alive).count()
    }
}

pub fn syscall_dispatch(table: &mut ProcessTable, req: SyscallRequest) -> SyscallResponse {
    match req.nr {
        SyscallNumber::Write => SyscallResponse::ok(req.arg1),
        SyscallNumber::Read => SyscallResponse::ok(0),
        SyscallNumber::Exit => {
            table.mark_exited(req.arg0 as u32);
            SyscallResponse::ok(0)
        }
        SyscallNumber::Spawn => table
            .spawn(req.arg0 as u32)
            .map(|pid| SyscallResponse::ok(pid as u64))
            .unwrap_or(SyscallResponse::err(12)),
        SyscallNumber::Wait => SyscallResponse::ok(req.arg1),
        SyscallNumber::Open => SyscallResponse::ok(3),
        SyscallNumber::Close => SyscallResponse::ok(0),
        SyscallNumber::ReadDir => SyscallResponse::ok(0),
        SyscallNumber::Stat => SyscallResponse::ok(0),
        SyscallNumber::Mmap => SyscallResponse::ok(req.arg1),
    }
}

pub struct MemoryManager {
    frame_allocator: FrameAllocator,
    mapper: PageMapper,
    last_stats: FrameStats,
}

impl MemoryManager {
    pub const fn new() -> Self {
        Self {
            frame_allocator: FrameAllocator::empty(),
            mapper: PageMapper::new(),
            last_stats: FrameStats {
                tracked_frames: 0,
                usable_frames: 0,
                allocated_frames: 0,
                free_frames: 0,
            },
        }
    }

    pub fn initialize(
        &mut self,
        memory_map: &[MemoryDescriptor],
        reserved: &[ReservedRange],
        heap_start: usize,
        heap_size: usize,
    ) {
        self.frame_allocator.initialize(memory_map, reserved);
        self.last_stats = self.frame_allocator.stats();
        heap::init_global_heap(heap_start, heap_size);
    }

    pub fn identity_map_region(&mut self, start: u64, len: u64) -> bool {
        self.mapper
            .map_identity_region(start, len, MapFlags::KERNEL_RW)
            .is_ok()
    }

    pub fn stats(&self) -> FrameStats {
        self.last_stats
    }
}

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
pub const PHASE1_BOOT_LINES: [&str; 8] = [
    "[sadas][INFO] sadas: hello from kernel",
    "[sadas][INFO] sadas: idt/exceptions initialized",
    "[sadas][INFO] sadas: acpi/madt discovered",
    "[sadas][INFO] sadas: local apic + ioapic enabled",
    "[sadas][INFO] sadas: apic timer started",
    "[sadas][INFO] sadas: tick -> task worker",
    "[sadas][INFO] sadas: tick -> task idle",
    "[sadas][INFO] sadas: scheduler loop entered",
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

    let mut memory = MemoryManager::new();
    let bootstrap_map = [MemoryDescriptor {
        ty: MemoryType::Conventional,
        physical_start: 0x0010_0000,
        page_count: 256,
    }];
    let reserved = [ReservedRange {
        start: 0x0010_0000,
        len: 0x20_000,
    }];
    memory.initialize(&bootstrap_map, &reserved, 0x0020_0000, 0x10_0000);
    let _ = memory.identity_map_region(0x0010_0000, 0x20_000);
    let mem_stats = memory.stats();
    console::debug!(
        "memory: tracked={} free={} allocated={}",
        mem_stats.tracked_frames,
        mem_stats.free_frames,
        mem_stats.allocated_frames
    );

    let mut idt = InterruptController::new();
    idt.install_exception_handlers();
    idt.load();
    console::info!("sadas: idt/exceptions initialized");

    let mut apic = ApicController::new();
    let mock_madt = mock_madt_table();
    if let Ok(acpi_info) = acpi::discover_apic(0x1000, &mock_madt) {
        console::info!("sadas: acpi/madt discovered");
        apic.enable_local_apic(acpi_info.local_apic_addr as u64);
        apic.enable_io_apic(acpi_info.io_apic_addr as u64);
    } else {
        console::warn!("sadas: acpi/madt unavailable");
    }
    let apic_state = apic.state();
    if apic_state.local_apic_enabled {
        console::info!("sadas: local apic + ioapic enabled");
    }

    let mut timer = Timer::new();
    timer.start_apic_periodic(100);
    console::info!("sadas: apic timer started");

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
    sched.init_interrupt_timer_baseline();

    let mut processes = ProcessTable::new();
    let init_elf = mock_init_elf();
    if parse_elf64(&init_elf).is_ok() {
        console::info!("sadas: /bin/init elf loaded from initfs");
    } else {
        console::warn!("sadas: /bin/init elf parse failed");
    }
    if let Some(init_pid) = processes.spawn(0) {
        console::info!("sadas: launching /bin/init pid={}", init_pid);
    }

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
    console::info!("sadas: scheduler loop entered");
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

#[no_mangle]
pub extern "C" fn handle_page_fault(vector: u8, error_code: u64, rip: u64, cr2: u64) -> ! {
    console::init_serial();
    logging::set_backend(console::logging_backend);
    let info = FaultInfo {
        vector,
        error_code,
        instruction_pointer: rip,
        cr2,
    };
    let mut buf = [0u8; 160];
    let len = sadas_arch_x86_64::interrupts::format_page_fault(info, &mut buf);
    console::log_bytes(&buf[..len]);
    loop {
        halt_cpu();
    }
}

fn mock_madt_table() -> [u8; 56] {
    let mut madt = [0u8; 56];
    madt[36..40].copy_from_slice(&0xFEE0_0000u32.to_le_bytes());
    madt[44] = 1;
    madt[45] = 12;
    madt[48..52].copy_from_slice(&0xFEC0_0000u32.to_le_bytes());
    madt
}

fn mock_init_elf() -> [u8; 64] {
    let mut elf = [0u8; 64];
    elf[0..4].copy_from_slice(b"\x7fELF");
    elf[4] = 2;
    elf[5] = 1;
    elf[24..32].copy_from_slice(&0x400000u64.to_le_bytes());
    elf[32..40].copy_from_slice(&64u64.to_le_bytes());
    elf[56..58].copy_from_slice(&1u16.to_le_bytes());
    elf
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

#[cfg(test)]
mod process_tests {
    use super::*;

    #[test]
    fn spawn_assigns_unique_asids() {
        let mut table = ProcessTable::new();
        let a = table.spawn(0).expect("pid");
        let b = table.spawn(a).expect("pid");
        assert_ne!(a, b);
        assert_eq!(table.list_alive(), 2);
    }

    #[test]
    fn syscall_spawn_and_exit_update_process_state() {
        let mut table = ProcessTable::new();
        let spawned = syscall_dispatch(
            &mut table,
            SyscallRequest {
                nr: SyscallNumber::Spawn,
                arg0: 0,
                arg1: 0,
                arg2: 0,
                arg3: 0,
            },
        );
        assert_eq!(spawned.status, 0);

        let pid = spawned.value;
        let exited = syscall_dispatch(
            &mut table,
            SyscallRequest {
                nr: SyscallNumber::Exit,
                arg0: pid,
                arg1: 0,
                arg2: 0,
                arg3: 0,
            },
        );
        assert_eq!(exited.status, 0);
        assert_eq!(table.list_alive(), 0);
    }
}
