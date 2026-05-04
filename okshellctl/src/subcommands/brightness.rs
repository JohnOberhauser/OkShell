use crate::bus::bus_command;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum BrightnessCommands {
    /// Increase the brightness by 5 percent
    Up,
    /// Decrease the brightness by 5 percent
    Down,
}

pub async fn execute(command: BrightnessCommands) -> anyhow::Result<()> {
    match command {
        BrightnessCommands::Up => {
            bus_command("BrightnessUp").await?;
        }
        BrightnessCommands::Down => {
            bus_command("BrightnessDown").await?;
        }
    }
    Ok(())
}
