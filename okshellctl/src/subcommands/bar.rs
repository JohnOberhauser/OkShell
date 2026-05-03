use clap::Subcommand;
use crate::bus::bus_command;

#[derive(Subcommand, Debug)]
pub enum BarCommands {
    /// Toggle the top bar
    Top,
    /// Toggle the bottom bar
    Bottom,
    /// Toggle the left bar
    Left,
    /// Toggle the right bar
    Right,
    /// Toggle all bars
    ToggleAll,
    /// Reveal all bars
    RevealAll,
    /// Hide all bars
    HideAll,
}

pub async fn execute(command: BarCommands) -> anyhow::Result<()> {
    match command {
        BarCommands::Top    => { bus_command("BarToggleTop").await?; }
        BarCommands::Bottom => { bus_command("BarToggleBottom").await?; }
        BarCommands::Left   => { bus_command("BarToggleLeft").await?; }
        BarCommands::Right  => { bus_command("BarToggleRight").await?; }
        BarCommands::ToggleAll    => { bus_command("BarToggleAll").await?; }
        BarCommands::RevealAll    => { bus_command("BarRevealAll").await?; }
        BarCommands::HideAll      => { bus_command("BarHideAll").await?; }
    }
    Ok(())
}
