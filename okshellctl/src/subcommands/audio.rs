use crate::bus::bus_command;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AudioCommands {
    /// Increase the volume by 5 percent
    VolumeUp,
    /// Decrease the volume by 5 percent
    VolumeDown,
    /// Toggle mute
    Mute,
}

pub async fn execute(command: AudioCommands) -> anyhow::Result<()> {
    match command {
        AudioCommands::VolumeUp => {
            bus_command("VolumeUp").await?;
        }
        AudioCommands::VolumeDown => {
            bus_command("VolumeDown").await?;
        }
        AudioCommands::Mute => {
            bus_command("Mute").await?;
        }
    }
    Ok(())
}
