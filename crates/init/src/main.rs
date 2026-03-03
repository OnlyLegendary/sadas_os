use sadas_kernel::{capability::Capability, scheduler::DeviceTier, scheduler::TaskId, Kernel};
use sadas_services::default_policies;
use sadas_ui::{Surface, UiProfile};

fn main() {
    println!("Sadas OS init: privacy-first system bring-up");

    let mut kernel = Kernel::new();
    kernel.set_device_tier(DeviceTier::Legacy);
    println!("runtime target: {}Hz", kernel.runtime_budget_hz());

    kernel.spawn_task(TaskId(1), "vaultd", 255);
    kernel.spawn_task(TaskId(2), "perm-broker", 240);
    kernel.spawn_task(TaskId(3), "compositor", 200);

    kernel.grant_capability(TaskId(1), Capability::IpcReceive);
    kernel.grant_capability(TaskId(2), Capability::IpcSend);

    let ui = UiProfile::for_legacy_device();
    let primary_surface = Surface {
        width: 1280,
        height: 720,
    };

    println!(
        "ui profile: {:?}, frame budget {}ms",
        ui,
        primary_surface.frame_budget_ms(kernel.runtime_budget_hz())
    );

    for policy in default_policies() {
        println!("service policy: {:?}", policy);
    }
}
