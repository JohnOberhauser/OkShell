use okshell_common::watch;
use okshell_services::sys_info_service;
use relm4::{Component, ComponentSender};

pub fn spawn_cpu_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = sys_info_service();
    let cpu = service.cpu.clone();

    watch!(sender, [cpu.watch(),], |out| {
        let _ = out.send(map_state());
    });
}

pub fn spawn_memory_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = sys_info_service();
    let memory = service.memory.clone();

    watch!(sender, [memory.watch(),], |out| {
        let _ = out.send(map_state());
    });
}

pub fn spawn_disk_watcher<C>(
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = sys_info_service();
    let disks = service.disks.clone();

    watch!(sender, [disks.watch(),], |out| {
        let _ = out.send(map_state());
    });
}
