use relm4::{Component, ComponentSender};
use okshell_common::{watch};
use okshell_idle::inhibitor::IdleInhibitor;

pub fn spawn_idle_inhibitor_watcher<C>(
    sender: &ComponentSender<C>,
    map: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
)
where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let inhibitor = IdleInhibitor::global();
    
    watch!(
        sender, 
        [
            inhibitor.watch(),
        ], 
        |out| {
            let _ = out.send(map());
        }
    );
}