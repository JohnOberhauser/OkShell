use crate::bus::{bus_command, bus_command_with_arg};
use clap::Subcommand;

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
    ToggleAll {
        /// Exclude bars that are hidden by default
        #[arg(short = 'x', long = "exclude")]
        exclude_hidden_by_default: bool,
    },
    /// Reveal all bars
    RevealAll {
        /// Exclude bars that are hidden by default
        #[arg(short = 'x', long = "exclude")]
        exclude_hidden_by_default: bool,
    },
    /// Hide all bars
    HideAll {
        /// Exclude bars that are hidden by default
        #[arg(short = 'x', long = "exclude")]
        exclude_hidden_by_default: bool,
    },
}

pub async fn execute(command: BarCommands) -> anyhow::Result<()> {
    match command {
        BarCommands::Top => {
            bus_command("BarToggleTop").await?;
        }
        BarCommands::Bottom => {
            bus_command("BarToggleBottom").await?;
        }
        BarCommands::Left => {
            bus_command("BarToggleLeft").await?;
        }
        BarCommands::Right => {
            bus_command("BarToggleRight").await?;
        }
        BarCommands::ToggleAll {
            exclude_hidden_by_default,
        } => {
            bus_command_with_arg("BarToggleAll", &exclude_hidden_by_default).await?;
        }
        BarCommands::RevealAll {
            exclude_hidden_by_default,
        } => {
            bus_command_with_arg("BarRevealAll", &exclude_hidden_by_default).await?;
        }
        BarCommands::HideAll {
            exclude_hidden_by_default,
        } => {
            bus_command_with_arg("BarHideAll", &exclude_hidden_by_default).await?;
        }
    }
    Ok(())
}
