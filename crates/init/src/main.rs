use sadas_kernel::{
    capability::Capability,
    ipc::Message,
    scheduler::{DeviceTier, TaskId},
    Kernel,
};
use sadas_services::default_policies;
use sadas_ui::{Surface, UiProfile};

fn main() {
    println!("Sadas OS init: polished full-stack bring-up");

    let mut kernel = Kernel::new();
    kernel.set_device_tier(DeviceTier::Balanced);

    kernel.spawn_task(TaskId(1), "vaultd", 255);
    kernel.spawn_task(TaskId(2), "perm-broker", 240);
    kernel.spawn_task(TaskId(3), "compositor", 230);
    kernel.spawn_task(TaskId(4), "shell", 220);
    kernel.spawn_task(TaskId(5), "compat-layer", 180);

    kernel.grant_capability(TaskId(2), Capability::IpcSend);
    kernel.grant_capability(TaskId(1), Capability::IpcReceive);
    kernel.grant_capability(TaskId(1), Capability::NetworkAccess);

    let ui = UiProfile::for_balanced_device();
    let surface = Surface {
        width: 1920,
        height: 1080,
    };

    println!(
        "runtime: {}Hz, background budget: {}",
        kernel.runtime_budget_hz(),
        kernel.max_background_tasks()
    );
    println!(
        "ui profile: {:?}, frame budget {}ms, render scale {}%",
        ui,
        surface.frame_budget_ms(kernel.runtime_budget_hz()),
        surface.recommended_render_scale()
    );

    for policy in default_policies() {
        println!("service policy: {:?}", policy);
    }

    let secure_message = Message {
        from: TaskId(2),
        to: TaskId(1),
        payload: [0; 64],
        secure_channel: true,
    };

    println!("secure IPC test: {:?}", kernel.send_message(secure_message));
}
