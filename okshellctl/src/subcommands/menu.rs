use crate::bus::bus_command;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum MenuCommands {
    /// Toggle the app launcher menu
    AppLauncher,
    /// Toggle the clipboard menu
    Clipboard,
    /// Toggle the clock menu
    Clock,
    /// Toggle the notifications menu
    Notifications,
    /// Toggle the quick settings menu
    QuickSettings,
    /// Toggle the screenshot menu
    Screenshot,
    /// Toggle the wallpaper menu
    Wallpaper,
    /// Close all open menus
    CloseAll,
}

pub async fn execute(command: MenuCommands) -> anyhow::Result<()> {
    match command {
        MenuCommands::QuickSettings => {
            bus_command("QuickSettings").await?;
        }
        MenuCommands::AppLauncher => {
            bus_command("AppLauncher").await?;
        }
        MenuCommands::Clipboard => {
            bus_command("Clipboard").await?;
        }
        MenuCommands::Clock => {
            bus_command("Clock").await?;
        }
        MenuCommands::Notifications => {
            bus_command("Notifications").await?;
        }
        MenuCommands::Screenshot => {
            bus_command("Screenshot").await?;
        }
        MenuCommands::Wallpaper => {
            bus_command("Wallpaper").await?;
        }
        MenuCommands::CloseAll => {
            bus_command("CloseAllMenus").await?;
        }
    }
    Ok(())
}
