use sadas_boot::BootPlan;
use sadas_drivers::default_driver_matrix;
use sadas_installer::InstallProfile;
use sadas_kernel::{
    capability::Capability,
    ipc::Message,
    scheduler::{CpuHint, DeviceTier, TaskId},
    Kernel,
};
use sadas_services::{default_policies, startup_plan};
use sadas_ui::{Surface, UiProfile};
use sadas_vm::VmProfile;

fn main() {
    println!("Sadas OS init: full operational bring-up profile");

    let boot = BootPlan::hardened();
    let vm = VmProfile::secure_default();
    let installer = InstallProfile::secure_desktop();
    println!("boot chain: {:?}", BootPlan::stage_order());
    println!("boot policy: {:?}", boot);
    for entry in BootPlan::qemu_boot_stub_report() {
        println!("boot stub: {:?} => {}", entry.stage, entry.message);
    }
    println!(
        "vm profile: {:?}, layout: {:?}",
        vm,
        VmProfile::baseline_layout()
    );
    println!(
        "installer profile: {:?}, steps: {:?}",
        installer,
        InstallProfile::steps()
    );

    let mut kernel = Kernel::new();
    kernel.set_device_tier(DeviceTier::Balanced);

    kernel.spawn_task(TaskId(1), "vaultd", 255, CpuHint::LatencySensitive);
    kernel.spawn_task(TaskId(2), "perm-broker", 240, CpuHint::LatencySensitive);
    kernel.spawn_task(TaskId(3), "compositor", 230, CpuHint::LatencySensitive);
    kernel.spawn_task(TaskId(4), "shell", 220, CpuHint::Balanced);
    kernel.spawn_task(TaskId(5), "compat-layer", 180, CpuHint::Background);
    kernel.spawn_task(TaskId(6), "syncd", 170, CpuHint::Background);
    kernel.spawn_task(TaskId(7), "updaterd", 160, CpuHint::Background);

    kernel.grant_capability(TaskId(2), Capability::IpcSend);
    kernel.grant_capability(TaskId(1), Capability::IpcReceive);
    kernel.grant_capability(TaskId(1), Capability::NetworkAccess);

    let ui = UiProfile::for_modern_device();
    let palette = ui.palette();
    let surface = Surface {
        width: 2560,
        height: 1440,
    };
    let layout = surface.desktop_layout(ui.density);

    println!(
        "runtime: {}Hz, background budget: {}, slice={}ms",
        kernel.runtime_budget_hz(),
        kernel.max_background_tasks(),
        kernel.interactive_slice_ms()
    );
    println!(
        "ui profile: {:?}, frame budget {}ms, render scale {}%",
        ui,
        surface.frame_budget_ms(kernel.runtime_budget_hz()),
        surface.recommended_render_scale()
    );
    println!(
        "palette bg=#{:06X} fg=#{:06X} accent=#{:06X} surface=#{:06X}",
        palette.background, palette.foreground, palette.accent, palette.surface
    );
    println!("desktop layout: {:?}", layout);

    let policies = default_policies();
    for policy in policies {
        println!("service policy: {:?}", policy);
    }
    println!("startup plan: {:?}", startup_plan(&policies));

    let drivers = default_driver_matrix();
    println!("driver matrix count: {}", drivers.len());

    let secure_message = Message {
        from: TaskId(2),
        to: TaskId(1),
        payload: [0; 64],
        secure_channel: true,
    };

    println!("secure IPC test: {:?}", kernel.send_message(secure_message));
}
