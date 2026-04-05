use clap::Subcommand;
use crate::bus::{bus_command, bus_command_with_reply};

#[derive(Subcommand, Debug)]
pub enum LockCommands {
    /// Lock the active session
    Activate,
    /// Check if the session is lock. Prints "locked" or "unlocked"
    Check,
}

pub async fn execute(command: LockCommands) -> anyhow::Result<()> {
    match command {
        LockCommands::Activate => { bus_command("Lock").await?; },
        LockCommands::Check => {
            let locked: bool = bus_command_with_reply("CheckLock").await?;
            println!("{}", if locked { "locked" } else { "unlocked" }); 
        },
    }
    Ok(())
}