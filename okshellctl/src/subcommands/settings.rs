use clap::Subcommand;
use crate::bus::bus_command;

#[derive(Subcommand, Debug)]
pub enum SettingsCommands {
    /// Open the settings window
    Open,
    /// Close the settings window
    Close,
}

pub async fn execute(command: SettingsCommands) -> anyhow::Result<()> {
    match command {
        SettingsCommands::Open => { bus_command("OpenSettings").await?; },
        SettingsCommands::Close => { bus_command("CloseSettings").await?; },
    }
    Ok(())
}